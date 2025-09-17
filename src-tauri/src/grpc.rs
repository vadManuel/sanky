use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Default)]
pub struct Streams(Arc<Mutex<HashMap<String, StreamHandle>>>);

struct StreamHandle {
    child: Child,
    stdin: Option<ChildStdin>,
    paused: Arc<AtomicBool>,
}

impl Streams {
    fn insert(&self, key: String, handle: StreamHandle) {
        self.0.lock().unwrap().insert(key, handle);
    }
    fn remove(&self, key: &str) -> Option<StreamHandle> {
        self.0.lock().unwrap().remove(key)
    }
    fn with<F, R>(&self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut StreamHandle) -> R,
    {
        let mut map = self.0.lock().unwrap();
        map.get_mut(key).map(f)
    }
}

#[derive(Debug, Deserialize)]
pub struct StreamingCallParams {
    pub address: String,
    pub method: String, // Service.Method or package.Service.Method
    pub request_data: Option<serde_json::Value>,
    pub streaming_data: Option<serde_json::Value>,
    pub proto_content: Option<String>,
    pub rpc_type: String, // 'server-streaming' | 'client-streaming' | 'bidirectional-streaming'
}

#[derive(Debug, Deserialize)]
pub struct StreamingSignalParams {
    pub address: String,
    pub method: String,
    pub signal: String, // 'cancel' | 'end' | 'pause' | 'resume'
}

#[derive(Debug, Deserialize)]
pub struct SendMessageParams {
    pub address: String,
    pub method: String,
    pub message: serde_json::Value,
}

// Removed unused StreamEventPayload

fn build_method_path(method: &str) -> String {
    if method.contains('/') || method.contains('.') {
        // If already in service/method or service.method, pass through
        method.to_string()
    } else {
        // Fallback: prefix a slash for bare names
        format!("/{}", method)
    }
}

fn write_temp_proto(proto: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("tauri_grpc_{}.proto", uuid::Uuid::new_v4()));
    std::fs::write(&path, proto).map_err(|e| format!("failed writing temp proto: {}", e))?;
    Ok(path)
}

pub async fn make_streaming_call(
    app: AppHandle,
    streams: Streams,
    params: StreamingCallParams,
) -> Result<(), String> {
    let insecure = true;
    let mut cmd = Command::new("grpcurl");
    if insecure {
        cmd.arg("-plaintext");
    }

    // If proto provided, write to temp and add flags
    let mut tmp_proto: Option<PathBuf> = None;
    if let Some(proto) = params.proto_content.as_ref() {
        let path = write_temp_proto(proto)?;
        cmd.arg("-import-path").arg(path.parent().unwrap());
        cmd.arg("-proto").arg(&path);
        tmp_proto = Some(path);
    }

    // Always use -d @ to feed JSON via stdin
    cmd.arg("-d").arg("@");
    cmd.arg(&params.address);
    cmd.arg(build_method_path(&params.method));
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn grpcurl: {}", e))?;

    // Obtain stdin to write initial messages if needed
    let mut stdin = child.stdin.take();

    // For server-streaming, write request_data then close stdin
    // For client/bidi, optionally write initial message, keep stdin open
    if let Some(ref mut s) = stdin {
        match params.rpc_type.as_str() {
            "server-streaming" => {
                if let Some(body) = params.request_data.as_ref() {
                    let payload =
                        serde_json::to_vec(body).map_err(|e| format!("invalid JSON: {}", e))?;
                    s.write_all(&payload)
                        .map_err(|e| format!("stdin write failed: {}", e))?;
                } else {
                    s.write_all(b"{}\n")
                        .map_err(|e| format!("stdin write failed: {}", e))?;
                }
                // Close stdin to signal end of request body
                // drop below
            }
            "client-streaming" | "bidirectional-streaming" => {
                if let Some(body) = params.streaming_data.as_ref() {
                    let payload =
                        serde_json::to_vec(body).map_err(|e| format!("invalid JSON: {}", e))?;
                    s.write_all(&payload)
                        .map_err(|e| format!("stdin write failed: {}", e))?;
                    s.write_all(b"\n").ok();
                    s.flush().ok();
                }
            }
            _ => {}
        }
    }

    // For server-streaming, close stdin now
    let stdin_to_store = match params.rpc_type.as_str() {
        "server-streaming" => None,
        _ => stdin,
    };

    let key = format!("{}-{}", params.address, params.method);

    // Move stdout handle out for reading
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "missing stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "missing stderr".to_string())?;

    // Insert stream handle before spawning read task
    let paused_flag = Arc::new(AtomicBool::new(false));
    streams.insert(
        key.clone(),
        StreamHandle {
            child,
            stdin: stdin_to_store,
            paused: paused_flag.clone(),
        },
    );

    // Spawn a thread to read stdout lines and emit events
    let app_for_out = app.clone();
    let tmp_for_out = tmp_proto.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut buf = String::new();
        let mut depth: i32 = 0;
        let mut in_string = false;
        let mut escape = false;

        for line in reader.lines() {
            match line {
                Ok(l) => {
                    if paused_flag.load(Ordering::SeqCst) {
                        continue;
                    }

                    // accumulate characters, tracking JSON string and brace depth
                    for ch in l.chars() {
                        if in_string {
                            if escape {
                                escape = false;
                            } else if ch == '\\' {
                                escape = true;
                            } else if ch == '"' {
                                in_string = false;
                            }
                        } else if ch == '"' {
                            in_string = true;
                        } else if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                        }
                        buf.push(ch);
                    }
                    buf.push('\n');

                    if depth <= 0 {
                        let trimmed = buf.trim();
                        if !trimmed.is_empty() {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                let _ = app_for_out.emit("streaming-data", json);
                            } else {
                                let _ = app_for_out.emit("streaming-data", trimmed.to_string());
                            }
                        }
                        buf.clear();
                        depth = 0;
                        in_string = false;
                        escape = false;
                    }
                }
                Err(e) => {
                    let _ = app_for_out.emit("streaming-error", format!("read error: {}", e));
                    break;
                }
            }
        }

        let trimmed = buf.trim();
        if !trimmed.is_empty() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(trimmed) {
                let _ = app_for_out.emit("streaming-data", json);
            } else {
                let _ = app_for_out.emit("streaming-data", trimmed.to_string());
            }
        }

        let _ = app_for_out.emit("streaming-end", String::from("done"));
        if let Some(p) = tmp_for_out {
            let _ = std::fs::remove_file(p);
        }
    });

    // Also capture and emit stderr lines as errors
    let app_for_err = app.clone();
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(l) = line {
                let _ = app_for_err.emit("streaming-error", l);
            }
        }
    });

    Ok(())
}

pub async fn send_streaming_signal(
    streams: Streams,
    params: StreamingSignalParams,
) -> Result<(), String> {
    let key = format!("{}-{}", params.address, params.method);
    match params.signal.as_str() {
        "cancel" => {
            if let Some(mut handle) = streams.remove(&key) {
                let _ = handle.child.kill();
            }
        }
        "end" => {
            streams.with(&key, |h| h.stdin.take());
        }
        "pause" => {
            streams.with(&key, |h| {
                h.paused.store(true, Ordering::SeqCst);
                ()
            });
        }
        "resume" => {
            streams.with(&key, |h| {
                h.paused.store(false, Ordering::SeqCst);
                ()
            });
        }
        other => return Err(format!("unknown signal: {}", other)),
    }
    Ok(())
}

pub async fn send_streaming_message(
    streams: Streams,
    params: SendMessageParams,
) -> Result<(), String> {
    let key = format!("{}-{}", params.address, params.method);
    let payload =
        serde_json::to_vec(&params.message).map_err(|e| format!("invalid JSON: {}", e))?;
    let wrote = streams.with(&key, |h| {
        if let Some(stdin) = h.stdin.as_mut() {
            if let Err(_e) = stdin.write_all(&payload) {
                return false;
            }
            let _ = stdin.write_all(b"\n");
            let _ = stdin.flush();
            true
        } else {
            false
        }
    });
    if wrote == Some(true) {
        Ok(())
    } else {
        Err("no active stream or stdin closed".into())
    }
}
