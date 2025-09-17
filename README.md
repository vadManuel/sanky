# sanky – Self‑hostable API platform (desktop)

An open‑source, self‑hostable API platform focused on gRPC first. This desktop app lets developers import `.proto` files or use server reflection to explore services, craft requests, and inspect responses in a clean, modern UI.

## Tech stack

- Frontend: Vue 3 + TypeScript, Element Plus, Pinia, Vue Router, Vite
- Desktop shell: Tauri (Rust)
- Backend bridge: Rust commands invoking `grpcurl` for gRPC (unary and streaming)

## Prerequisites

- Node.js and pnpm
- Rust toolchain (cargo) for Tauri
- Tauri system deps (see Tauri docs for your OS)
- grpcurl available on PATH

## Run

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm install
pnpm tauri build
```

## Notes

- For servers without reflection, paste/import a `.proto`; the app writes a temp file for `grpcurl` and cleans it up after calls/streams.
- Use fully‑qualified method names under the hood (package.Service.Method). The UI resolves this from your `.proto` automatically.
