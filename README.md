# mir

[![Docs](https://img.shields.io/badge/docs-jorgsowa.github.io%2Fmir-blue)](https://jorgsowa.github.io/mir/)

> ⚠️ **Experimental.** mir is under active development and not yet production-ready. APIs, CLI flags, issue codes, and output formats may change between releases; expect false positives and rough edges.

A fast, incremental PHP static analyzer written in Rust, inspired by [Psalm](https://psalm.dev).

## Features

- Sound type system — scalars, objects, generics, unions, intersections, literals, `never`, `void`
- Full type inference — return types inferred from bodies; types narrowed through `if`/`match`/`instanceof`/`is_string()` etc.
- Call checking — argument count and types for user-defined and built-in functions/methods
- Class analysis — inheritance, interface compliance, abstract enforcement, visibility, `readonly`, `final`
- Dead code detection — unused private methods, properties, functions
- Taint analysis — tracks data from `$_GET`/`$_POST` to HTML/SQL/shell sinks
- Incremental cache — unchanged files skipped on re-runs via SHA-256 content hashing
- Parallel analysis — rayon-powered; scales to available CPUs
- Comprehensive built-in coverage — powered by [JetBrains phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) (500+ functions, 100+ classes, 200+ constants across 33 PHP extensions)

## Installation

### From Composer (PHP projects)

```bash
composer require --dev open-mir/analyzer
vendor/bin/mir src/
```

A `post-install-cmd` hook downloads the prebuilt binary matching your version
and host platform from GitHub Releases. See
[docs/getting-started.md](docs/getting-started.md#installation) for supported
targets.

### From crates.io

```bash
cargo install mir-php
```

### Build from source

```bash
git clone --recurse-submodules https://github.com/jorgsowa/mir.git
cd mir
cargo build --release
# binary at target/release/mir
```

> **Note:** The `--recurse-submodules` flag is required to initialize the
> [phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) submodule that
> provides PHP built-in definitions. If you cloned without it, run:
> ```bash
> git submodule update --init
> ```

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
| [docs/getting-started.md](docs/getting-started.md) | Installation, first run, understanding output |
| [docs/configuration.md](docs/configuration.md) | `mir.xml` reference, baselines, CI setup |
| [docs/cli.md](docs/cli.md) | All flags, output formats, exit codes |
| [docs/issue-kinds.md](docs/issue-kinds.md) | Every issue type mir can emit |
| [docs/docblock.md](docs/docblock.md) | Supported docblock annotations |
| [docs/architecture.md](docs/architecture.md) | Crate layout and analysis pipeline |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## What's next

- Literal equality narrowing (`$x === 'foo'` → `TLiteralString`)
- `UnusedVariable` / `UnusedParam` detection
- Reduce `UndefinedMethod` / `InvalidArgument` false positives
- `PossiblyUndefinedVariable` detection
- Plugin system
- PHP version–aware stub filtering (load only stubs valid for the target version)

## License

MIT
