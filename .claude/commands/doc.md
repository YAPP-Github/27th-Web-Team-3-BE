---
description: Generate and open project documentation
---

# Documentation

Generates documentation from code comments and opens it in the browser.

## Usage

```bash
cd rust
cargo doc --no-deps --open
```

## Details

- `--no-deps`: Skips documenting dependencies (faster)
- `--open`: Opens the generated HTML in your default browser
- Useful for checking `utoipa` (Swagger) structs and function docs
