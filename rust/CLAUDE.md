# CLAUDE.md - Server Project Guide

## Build & Test Commands
- Full Build/Deploy: `./build.sh` (이미지에 있는 쉘 스크립트 활용)
- Standard Build: `cargo build`
- Run Server: `cargo run`
- Test All: `cargo test`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Project Context
- **Name:** Rust Server
- **Environment:** Configured via `.env` file
- **Build System:** Custom `build.sh` script used for deployments
- **Core Logic:** Located in `src/`

## Coding Style & Rules
- **Error Handling:** Use `Result` and `anyhow` for server-side errors.
- **Environment Variables:** Always use the `.env` patterns for configuration.
- **Async:** Use `tokio` patterns for asynchronous tasks (if applicable).
- **Documentation:** Public functions should have summary doc comments.
- **Strictness:** Follow `clippy` suggestions to maintain code quality.
- **TDD:** Prioritize writing tests in `src/` before implementing complex server logic.