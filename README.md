# Polsia

Polsia is a small experimental data language parser written in Rust using the [chumsky](https://github.com/zesterer/chumsky) parser combinator library.

## Language features

- `#` comments
- trailing commas in arrays and objects
- unquoted identifiers as keys
- optional commas and braces for single objects
- chained keys like `foo: bar: 1` for nested objects
- basic type annotations (`Int`, `Float`, `String`, `Any`, `Nothing`)

## Examples

```polsia
# simple object without braces
foo: 1
bar: [1, 2, 3,]
```

```polsia
# using types and chains
person: {
  name: String,
}
person: name: "Jane"
```

## Building

Use Cargo to build and run:

```bash
cargo run <path-to-file>
```

## Testing

Run the formatter, lints and tests:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

### Web playground

Build the WebAssembly package using [wasm-pack](https://github.com/rustwasm/wasm-pack):

```bash
wasm-pack build --target web --release -- --features wasm
```

Open `index.html` in a browser to try the playground.
