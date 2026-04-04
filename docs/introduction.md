# mir

A fast, incremental PHP static analyzer written in Rust, inspired by [Psalm](https://psalm.dev).

## Quick start

```bash
cargo install --path crates/mir-cli
mir src/
```

## What mir checks

- **Undefined** — variables, functions, methods, classes, properties, constants
- **Type mismatches** — argument types, return types, property assignments
- **Nullability** — null dereferences, nullable returns
- **Dead code** — unused private methods, properties, functions
- **Inheritance** — abstract methods, interface compliance, signature compatibility
- **Security** — taint tracking from `$_GET`/`$_POST` to HTML/SQL/shell sinks
- **Generics** — `@template` bounds and substitution

mir understands ~580 PHP built-in functions and produces output compatible with
Psalm baselines (`--baseline psalm-baseline.xml`).
