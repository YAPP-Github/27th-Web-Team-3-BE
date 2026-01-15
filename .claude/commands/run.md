---
description: Run the Rust API server locally
---

# Run Server

Executes the Rust backend server using `cargo`.

## Usage

```bash
cd codes
cargo run -p web3-server
```

## Options

- To run with a specific profile: `cargo run --release`
- To run with specific environment variables: `RUST_LOG=debug cargo run`
