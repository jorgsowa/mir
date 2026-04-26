# Contributing to mir stubs

## Two-layer architecture

Stubs load in order — later layers override earlier ones:

1. **phpstorm-stubs** (`crates/mir-analyzer/phpstorm-stubs/`) — the authoritative
   JetBrains community stubs. Covers the full PHP standard library.
2. **Custom stubs** (`stubs/{ext}/*.php`) — precise type overrides for functions
   where phpstorm-stubs is too broad (e.g. wrong return type, missing by-ref param).

## When to add to STUB_DIRS vs. write custom stubs

| Situation | Action |
|---|---|
| Extension exists in phpstorm-stubs submodule | Add directory name to `STUB_DIRS` in `crates/mir-analyzer/build.rs` |
| Extension is absent from phpstorm-stubs, or types are imprecise | Write custom stubs under `stubs/{ext}/` |

Check whether the directory exists before adding to `STUB_DIRS`:

```sh
ls crates/mir-analyzer/phpstorm-stubs/
```

## stub.toml format

Every custom extension directory requires a `stub.toml`:

```toml
[extension]
name        = "apcu"          # PHP extension name (informational)
version     = "5.1.0"         # Minimum extension version these stubs target
php-min     = "7.0"           # Minimum PHP version
composer    = "ext-apcu"      # Composer platform requirement
description = "APC User Cache"
```

All fields are required. The file is not parsed at runtime — it serves as
documentation for maintainers.

## @since/@removed filtering

The `@since` and `@removed` tags in stub docblocks are respected when loading
stubs for a specific PHP version. Symbols are filtered at the following levels:

- Top-level functions (`function foo() {}`)
- Classes, interfaces, traits, enums
- Methods (including interface methods)
- Class/interface/trait constants (`ClassConstDecl`)
- Class/trait properties (`PropertyDecl`)
- Global constants (`const FOO = 1;` and `define('FOO', 1)`)

**Currently not filtered:** enum cases.

Example docblock:

```php
/**
 * @since 8.1
 */
function array_is_list(array $array): bool {}
```

## Writing a .phpt fixture test

Fixture tests live in `crates/mir-analyzer/tests/fixtures/{category}/`.
Each file is a plain-text `.phpt` with sections separated by `---`:

```
--TEST--
Custom stub function is recognized

--FILE--
<?php
$result = apcu_store('key', 'value');

--EXPECT--
```

Run all tests with:

```sh
cargo test -p mir-analyzer
```

## Rebuilding

After changing `build.rs` or adding stub files, rebuild to re-embed:

```sh
cargo build -p mir-analyzer
```
