# Polsia

Polsia is a small experimental data language parser written in Rust using the [chumsky](https://github.com/zesterer/chumsky) parser combinator library.

## Language features

- `#` comments
- trailing commas in arrays and objects
- unquoted identifiers as keys
- optional commas and braces for single objects
- chained keys like `foo: bar: 1` for nested objects
- basic type annotations (`Int`, `Float`, `String`, `Boolean`, `Any`, `Nothing`)

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

## Testing

Run the formatter, lints and tests:

```bash
just test
```

### Web playground

The playground is a small React application in the `playground/` directory built with [Vite](https://vitejs.dev/).
To run it locally:

```bash
just playground dev
```

The site will be available at the URL printed by Vite (usually `http://localhost:5173/polsia/`).
