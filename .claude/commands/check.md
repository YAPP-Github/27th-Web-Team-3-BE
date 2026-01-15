---
description: Quickly check code for compilation errors
---

# Fast Check

Performs a fast compilation check without generating the final binary. Use this frequently during development to catch type errors early.

## Usage

```bash
cd rust
cargo check
```

## Benefits

- Significantly faster than `cargo build`
- Catches 99% of syntax and type errors
- essential for the "Compiler Feedback Loop"
