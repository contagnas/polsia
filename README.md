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

The playground is a small React application in the `playground/` directory built with [Vite](https://vitejs.dev/).
To run it locally:

```bash
# build the WebAssembly package
wasm-pack build --target bundler --release -- --features wasm

# install dependencies
cd playground
npm install

# run frontend tests
npm test

# start the dev server
npm run dev

# build the static site
npm run build
```

The site will be available at the URL printed by Vite (usually `http://localhost:5173`).
