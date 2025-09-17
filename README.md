# Tauri + Vue + TypeScript

This template should help get you started developing with Vue 3 and TypeScript in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## gRPC Client (minimal)

This app includes a minimal gRPC client UI backed by Rust commands:

- `grpc_list_services(address: String)` lists services via server reflection (plaintext).
- `grpc_invoke_unary({ address, full_method, request_json, proto_content?, insecure? })` invokes a unary RPC using `grpcurl` under the hood.

Notes:

- Requires `grpcurl` installed and available on PATH.
- By default, calls are plaintext (`-plaintext`). Provide `.proto` content if the target server does not expose reflection. The `.proto` is written to a temp file for the call and removed.
- The Vue UI is intentionally minimal: enter `address`, list services, choose service/method, and send a JSON request. Response displays as JSON.
