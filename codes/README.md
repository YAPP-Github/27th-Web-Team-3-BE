# Rust Server

A simple HTTP server built with Rust using the Axum web framework.

## Features

- **Fast and Async**: Built with Tokio runtime for high-performance async I/O
- **Modern Web Framework**: Uses Axum for routing and handlers
- **CORS Enabled**: Configured for cross-origin requests
- **Logging**: Integrated tracing for request logging

## Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

## Installation

1. Install Rust from [rustup.rs](https://rustup.rs/) if you haven't already
2. Clone this repository
3. Install dependencies:
   ```bash
   cargo build
   ```

## Running the Server

```bash
cargo run
```

The server will start on `http://127.0.0.1:3000`

## API Endpoints

### GET /
- **Description**: Root endpoint
- **Response**: Welcome message

### GET /health
- **Description**: Health check endpoint
- **Response**: JSON with status and timestamp
```json
{
  "status": "healthy",
  "timestamp": "2026-01-15T12:00:00Z"
}
```

### POST /api/echo
- **Description**: Echo endpoint that returns the message sent
- **Request Body**:
```json
{
  "message": "Hello, World!"
}
```
- **Response**:
```json
{
  "echo": "Hello, World!"
}
```

## Development

### Build for Release
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Check Code
```bash
cargo check
```

### Format Code
```bash
cargo fmt
```

### Lint Code
```bash
cargo clippy
```

## Project Structure

```
.
├── Cargo.toml      # Project dependencies and metadata
├── src/
│   └── main.rs     # Main application entry point
├── .gitignore      # Git ignore rules
└── README.md       # This file
```

## Dependencies

- **axum**: Web framework
- **tokio**: Async runtime
- **serde**: Serialization/deserialization
- **tower-http**: HTTP middleware (CORS)
- **tracing**: Logging framework
- **chrono**: Date and time handling

## License

MIT

