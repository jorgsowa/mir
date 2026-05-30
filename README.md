<div align="center">

<img src="site/src/assets/logo-hero.png" width="120" alt="mir">

</div>

# mir

[![Docs](https://img.shields.io/badge/docs-jorgsowa.github.io%2Fmir-blue)](https://jorgsowa.github.io/mir/)

> ⚠️ **Experimental.** mir is under active development and not yet production-ready. APIs, CLI flags, issue codes, and output formats may change between releases; expect false positives and rough edges.

**[Try the Playground →](https://jorgsowa.github.io/mir/playground)**

A fast, incremental PHP static analyzer written in Rust, inspired by [Psalm](https://psalm.dev).

## Features

- **77 diagnostic rules** across type errors, undefined symbols, dead code, taint, and more
- Sound type system — scalars, objects, generics, unions, intersections, literals, `never`, `void`
- Full type inference — return types, literal narrowing, `if`/`match`/`instanceof`/`is_string()` etc.
- Call checking — argument count and types for user-defined and built-in functions/methods
- Class analysis — inheritance, interface compliance, abstract enforcement, visibility, `readonly`, `final`
- Dead code detection — unused variables, parameters, private methods, properties, and functions
- Taint analysis — tracks data from `$_GET`/`$_POST` to HTML/SQL/shell sinks
- Incremental cache — unchanged files skipped on re-runs via SHA-256 content hashing
- Parallel analysis — rayon-powered; scales to available CPUs
- PHP 7.4–8.5 support with version-aware stub filtering
- Comprehensive built-in coverage — powered by [JetBrains phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) (57 extensions, 500+ functions, 100+ classes)

## Installation

### From Composer (PHP projects)

```bash
composer require --dev miropen/mir-php
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
git clone https://github.com/jorgsowa/mir.git
cd mir
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

### Suppressing issues inline

Silence a single false positive directly in the source with a comment, without
editing config or a baseline file:

```php
// @mir-ignore UndefinedClass
new ThirdPartyClass();

$obj->method();          // @mir-ignore NullMethodCall   (trailing: this line)

// @mir-ignore-next-line PossiblyNullArgument
doThing($maybeNull);
```

A bare comment above a statement annotates the statement that follows it; a
trailing comment annotates its own line. List one or more `IssueKind` names (or
codes such as `MIR1400`) after the directive, or omit them to suppress every
issue on the target line. `@mir-ignore-file Kind …` suppresses a kind across the
whole file. The `@psalm-suppress`, `@suppress`, `@phpstan-ignore-line` and
`@phpstan-ignore-next-line` aliases are accepted for drop-in compatibility.

## Documentation

Full documentation is available at **[jorgsowa.github.io/mir](https://jorgsowa.github.io/mir/)**.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
