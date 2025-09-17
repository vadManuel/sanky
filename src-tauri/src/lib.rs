mod grpc;
use grpc::{make_streaming_call, send_streaming_message, send_streaming_signal, Streams};
use regex::Regex;
use tauri::State;
#[cfg(any(target_os = "ios", target_os = "android"))]
compile_error!("Mobile builds are disabled for this app.");

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn grpc_list_services(address: String) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("grpcurl")
        .arg("-plaintext")
        .arg(address)
        .arg("list")
        .output()
        .map_err(|e| format!("failed to spawn grpcurl: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let services = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(services)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("grpcurl error: {}", stderr))
    }
}

#[derive(serde::Deserialize)]
struct GrpcInvokeParams {
    address: String,
    full_method: String,
    request_json: serde_json::Value,
    proto_content: Option<String>,
    insecure: Option<bool>,
}

#[tauri::command]
async fn grpc_invoke_unary(params: GrpcInvokeParams) -> Result<serde_json::Value, String> {
    let insecure = params.insecure.unwrap_or(true);

    let mut cmd = std::process::Command::new("grpcurl");
    if insecure {
        cmd.arg("-plaintext");
    }

    let tmpdir = std::env::temp_dir();
    let mut tmp_proto_path: Option<std::path::PathBuf> = None;
    if let Some(proto) = params.proto_content.as_ref() {
        let file_path = tmpdir.join(format!("tauri_grpc_{}.proto", uuid::Uuid::new_v4()));
        if let Err(e) = std::fs::write(&file_path, proto) {
            return Err(format!("failed writing temp proto: {}", e));
        }
        cmd.arg("-import-path").arg(&tmpdir);
        cmd.arg("-proto").arg(&file_path);
        tmp_proto_path = Some(file_path);
    }

    let json_payload = match serde_json::to_vec(&params.request_json) {
        Ok(v) => v,
        Err(e) => return Err(format!("invalid request json: {}", e)),
    };

    cmd.arg("-d").arg("@");
    cmd.arg(&params.address);

    let method_path = if params.full_method.contains('/') || params.full_method.contains('.') {
        // Accept already well-formed service/method or service.method
        params.full_method.clone()
    } else {
        // Backwards compatibility: prefix a slash if only a bare name is provided
        format!("/{}", params.full_method)
    };
    cmd.arg(method_path);

    let mut child = match cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return Err(format!("failed to spawn grpcurl: {}", e)),
    };

    use std::io::Write as _;
    if let Some(stdin) = child.stdin.as_mut() {
        if let Err(e) = stdin.write_all(&json_payload) {
            return Err(format!("failed writing to grpcurl stdin: {}", e));
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => return Err(format!("failed to read grpcurl output: {}", e)),
    };

    if let Some(path) = tmp_proto_path {
        let _ = std::fs::remove_file(path);
    }

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        match serde_json::from_str::<serde_json::Value>(&stdout) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!(
                "failed to parse grpcurl json: {}\nRaw: {}",
                e, stdout
            )),
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("grpcurl error: {}", stderr))
    }
}

pub fn run() {
    tauri::Builder::default()
        .manage(Streams::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            grpc_list_services,
            grpc_invoke_unary,
            grpc_make_streaming_call,
            grpc_send_streaming_signal,
            grpc_send_streaming_message,
            select_and_read_file,
            read_file_at_path,
            reflect_list_services,
            reflect_describe_service,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn grpc_make_streaming_call(
    app: tauri::AppHandle,
    streams: State<'_, Streams>,
    params: grpc::StreamingCallParams,
) -> Result<(), String> {
    make_streaming_call(app, streams.inner().clone(), params).await
}

#[tauri::command]
async fn grpc_send_streaming_signal(
    streams: State<'_, Streams>,
    params: grpc::StreamingSignalParams,
) -> Result<(), String> {
    send_streaming_signal(streams.inner().clone(), params).await
}

#[tauri::command]
async fn grpc_send_streaming_message(
    streams: State<'_, Streams>,
    params: grpc::SendMessageParams,
) -> Result<(), String> {
    send_streaming_message(streams.inner().clone(), params).await
}

#[tauri::command]
async fn select_and_read_file() -> Result<(String, String), String> {
    // Use plugin on frontend for picking, backend only reads path
    Err("not implemented: use read_file_at_path instead".into())
}

#[tauri::command]
async fn read_file_at_path(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("read failed: {}", e))
}

#[tauri::command]
async fn reflect_list_services(address: String) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("grpcurl")
        .arg("-plaintext")
        .arg(address)
        .arg("list")
        .output()
        .map_err(|e| format!("failed to spawn grpcurl: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

#[derive(serde::Serialize)]
struct MethodSig {
    name: String,
    input: String,
    output: String,
    streaming: String, // unary | server-streaming | client-streaming | bidirectional-streaming
}

#[tauri::command]
async fn reflect_describe_service(
    address: String,
    service: String,
) -> Result<Vec<MethodSig>, String> {
    let output = std::process::Command::new("grpcurl")
        .arg("-plaintext")
        .arg("-format")
        .arg("verbose")
        .arg(address)
        .arg("describe")
        .arg(&service)
        .output()
        .map_err(|e| format!("failed to spawn grpcurl: {}", e))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse lines like: rpc Method (pkg.Message) returns (pkg.Message)
    let mut methods = Vec::new();
    let re = Regex::new(r"^\s*rpc\s+(\w+)\s*\(\s*(stream\s+)?([\w\.]+)\s*\)\s*returns\s*\(\s*(stream\s+)?([\w\.]+)\s*\)").unwrap();
    for line in stdout.lines() {
        if let Some(caps) = re.captures(line) {
            let name = caps.get(1).unwrap().as_str().to_string();
            let in_stream = caps
                .get(2)
                .map(|m| m.as_str())
                .unwrap_or("")
                .contains("stream");
            let input = caps.get(3).unwrap().as_str().to_string();
            let out_stream = caps
                .get(4)
                .map(|m| m.as_str())
                .unwrap_or("")
                .contains("stream");
            let output = caps.get(5).unwrap().as_str().to_string();
            let streaming = if in_stream && out_stream {
                "bidirectional-streaming"
            } else if out_stream {
                "server-streaming"
            } else if in_stream {
                "client-streaming"
            } else {
                "unary"
            };
            methods.push(MethodSig {
                name,
                input,
                output,
                streaming: streaming.to_string(),
            });
        }
    }
    Ok(methods)
}
