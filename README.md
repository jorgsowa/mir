# mir

A fast, incremental PHP static analyzer written in Rust, inspired by [Psalm](https://psalm.dev).

## Features

- Sound type system — scalars, objects, generics, unions, intersections, literals, `never`, `void`
- Full type inference — return types inferred from bodies; types narrowed through `if`/`match`/`instanceof`/`is_string()` etc.
- Call checking — argument count and types for user-defined and ~580 built-in functions/methods
- Class analysis — inheritance, interface compliance, abstract enforcement, visibility, `readonly`, `final`
- Dead code detection — unused private methods, properties, functions
- Taint analysis — tracks data from `$_GET`/`$_POST` to HTML/SQL/shell sinks
- Incremental cache — unchanged files skipped on re-runs via SHA-256 content hashing
- Parallel analysis — rayon-powered; scales to available CPUs

## Installation

```bash
cargo install --path crates/mir-cli
```

Or build from source:

```bash
cargo build --release
# binary at target/release/mir
```

## Usage

```bash
mir                        # analyze current directory
mir src/ lib/              # analyze specific paths
mir --format json src/     # machine-readable output
mir --baseline baseline.xml src/  # suppress known issues
```

See [docs/cli.md](docs/cli.md) for the full CLI reference.

## Documentation

| Document | Contents |
|----------|----------|
| [docs/cli.md](docs/cli.md) | All flags, output formats, exit codes |
| [docs/issue-kinds.md](docs/issue-kinds.md) | Every issue type mir can emit |
| [docs/docblock.md](docs/docblock.md) | Supported docblock annotations |
| [docs/architecture.md](docs/architecture.md) | Crate layout and analysis pipeline |

## Roadmap

See [ROADMAP.md](ROADMAP.md) for the full milestone list.

**What's next:**
- Literal equality narrowing (`$x === 'foo'` → `TLiteralString`)
- `UnusedVariable` / `UnusedParam` detection
- Reduce `UndefinedMethod` / `InvalidArgument` false positives
- `PossiblyUndefinedVariable` detection
- Plugin system

## License

MIT
