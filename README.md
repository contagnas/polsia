# Aislop

Aislop is a small experimental JSON parser written in Rust using the [chumsky](https://github.com/zesterer/chumsky) parser combinator library.

Features include:
- Comments starting with `#`
- Trailing commas in arrays and objects
- Unquoted identifiers as object keys

## Building

Use Cargo to build and run:

```bash
cargo run <path-to-json-file>
```

## Testing

Run the formatter and tests:

```bash
cargo fmt -- --check
cargo test
```

