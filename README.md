<div align="center">

<img src="site/src/assets/logo-hero.png" width="120" alt="mir">

</div>

# mir

[![Crates.io](https://img.shields.io/crates/v/mir-php)](https://crates.io/crates/mir-php)
[![Packagist](https://img.shields.io/packagist/v/miropen/mir-php)](https://packagist.org/packages/miropen/mir-php)
[![Docs](https://img.shields.io/badge/docs-jorgsowa.github.io%2Fmir-blue)](https://jorgsowa.github.io/mir/)

**[Try the Playground →](https://jorgsowa.github.io/mir/playground)**

A fast, incremental PHP static analyzer written in Rust. It finds type errors,
undefined symbols, dead code, injection vulnerabilities and many, many more
without running your code. [Why mir?](https://jorgsowa.github.io/mir/why/)

```text
$ mir src/
src/demo.php:16:6 error[MIR0201] InvalidArgument: Argument $s of greet() expects 'string', got '42'
src/demo.php:18:0 error[MIR0004] UndefinedMethod: Method User::getName() does not exist
src/demo.php:21:0 error[MIR0802] TaintedSql: Tainted SQL query — possible SQL injection
  = taint flow: $_GET['q'] (line 20) → $q (line 20)
```

## Features

- **196 diagnostic rules** with stable codes (`MIR0201`): type errors, undefined symbols, nullability, arrays, generics, purity, dead code, taint and more. See the [issue reference](https://jorgsowa.github.io/mir/reference/issues/).
- **Type system:** scalars, objects, generics and templates, unions, intersections, literals, array shapes, `class-string<T>`, conditional return types, `never`, `void`.
- **Inference and narrowing:** return types, literal narrowing, `if` / `match` / `switch` / `instanceof` / `is_*()` / `isset()` guards, assertion annotations, and functions whose body is an `instanceof` check.
- **Call checking:** argument count, types, named arguments and by-reference parameters for user code and built-ins.
- **Class analysis:** inheritance, interface compliance, abstract enforcement, visibility, `readonly`, `final`, `#[Override]`, traits and enums.
- **Compile errors:** code that parses but PHP rejects (`CompileError`: trait conflicts, enum rules, invalid `extends`/`implements`), reported with PHP's wording for the target version.
- **Dead code** (`--find-dead-code`): unused variables, parameters, functions, classes, interfaces, traits and enums, plus private methods, properties and constants. `@api` exempts public API.
- **Taint analysis:** follows request data (`$_GET`, `$_POST`, …) to HTML, SQL, shell, header, cookie, SSRF, LDAP, XPath, callable and LLM-prompt sinks, and prints the path it took.
- **PHP 7.4–8.6:** code is parsed and checked at the target version (`--php-version` or `<phpVersion>`), including PHP 8.6 partial application.
- **Fast and incremental:** parallel analysis over all CPUs; results are cached per file content hash, so unchanged files are skipped on re-runs.
- **Plugins:** native Rust plugins loaded as shared libraries.
- **Built-in coverage:** bundled [JetBrains phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) for 77 extensions (2,500+ functions, 500+ classes), filtered by PHP version.
- **CI output formats:** `text`, `json`, `github-actions`, `junit` and `sarif`.
- **Works with existing code:** reads `@psalm-*` and `@phpstan-*` annotations and suppressions as they are, plus Psalm config files, baselines and plugins. See [Migrating to mir](https://jorgsowa.github.io/mir/guides/migrating/).

## Installation

### Composer (PHP projects)

```bash
composer require --dev miropen/mir-php
vendor/bin/mir src/
```

A `post-install-cmd` hook downloads the prebuilt binary for your version and
platform from GitHub Releases and verifies its checksum. Binaries are published
for Linux (x86_64, aarch64), macOS (x86_64, Apple Silicon) and Windows (x86_64).
See the [getting started guide](https://jorgsowa.github.io/mir/guides/getting-started/).

### crates.io

```bash
cargo install mir-php
```

### From source

```bash
git clone https://github.com/jorgsowa/mir.git
cd mir
cargo build --release
# binary at target/release/mir
```

## Usage

```bash
mir                              # analyze the composer project in the current directory
mir src/ lib/                    # analyze specific paths
mir --php-version 8.2 src/       # check against a specific PHP version
mir --find-dead-code src/        # also report unused code
mir --no-taint src/              # skip taint analysis
mir --format sarif src/ > mir.sarif
```

mir picks up `mir.xml` from the current directory or any parent.
When a `composer.json` is found, the project files come from its autoload sections
and vendor code is loaded on demand. Exit code is `1` when any error is reported.

See the [CLI reference](https://jorgsowa.github.io/mir/reference/cli/) and the
[configuration guide](https://jorgsowa.github.io/mir/guides/configuration/).

### Baselines

Adopt mir on an existing codebase without fixing every issue first:

```bash
mir src/ --set-baseline       # record current issues in mir-baseline.xml
mir src/                      # report only new issues
mir src/ --update-baseline    # drop entries for issues you fixed
```

### Suppressing issues inline

Use `@mir-ignore`, `@mir-ignore-next-line`, or `@mir-ignore-file` to suppress a
false positive directly in source. `@psalm-suppress` and `@phpstan-ignore-*`
are accepted too. See the [suppression reference](https://jorgsowa.github.io/mir/reference/suppressions/).

### GitHub Actions

Use the [mir GitHub Action](https://github.com/jorgsowa/mir-action):

```yaml
- uses: jorgsowa/mir-action@v1
  with:
    path: src
```

## Crates

| Crate | Purpose |
|-------|---------|
| [`mir-php`](crates/mir-cli) | The `mir` command-line tool |
| [`mir-analyzer`](crates/mir-analyzer) | Analysis engine: batch and incremental (salsa) analysis sessions |
| [`mir-types`](crates/mir-types) | Type representation and operations |
| [`mir-codebase`](crates/mir-codebase) | Definition types for classes, functions and constants |
| [`mir-issues`](crates/mir-issues) | Issue kinds, codes and severities |
| [`mir-plugin`](crates/mir-plugin) | Plugin API and plugin loading |
| [`mir-plugin-example`](crates/mir-plugin-example) | Example Rust plugin |
| [`mir-wasm`](crates/mir-wasm) | WebAssembly build that powers the playground |

See the [architecture overview](https://jorgsowa.github.io/mir/reference/architecture/).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
