# AGENTS

## Guidelines for Codex

- Format all Rust code with `cargo fmt -- --check` before committing.
- Run `cargo test` and make sure all tests pass.
- Run `cargo clippy` and ensure there are no warnings.
- Run `cargo check --target wasm32-unknown-unknown --features wasm` to verify the
  WebAssembly build works.
- Include the output of these commands in the Testing section of pull request messages.

