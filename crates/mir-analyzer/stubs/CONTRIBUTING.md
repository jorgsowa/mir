# Contributing to mir stubs

## Layout

Each directory under `stubs/` holds the stubs for one PHP extension. Most are
copied from [JetBrains phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs);
directories with a `stub.toml` are written for mir.

At compile time `build.rs` embeds every `.php` file under `stubs/` and reads
`PhpStormStubsMap.php`, which maps each function, class and constant to the
file that declares it. Name lookups go through that map, so every new symbol
needs an entry there.

## Changing stubs

| Situation | Action |
|---|---|
| Upstream type is wrong or too broad | Edit the vendored file in place; keep the change minimal so upstream syncs stay easy |
| Extension exists upstream but isn't vendored | Copy its directory from phpstorm-stubs |
| Extension is absent upstream | Add a directory with a `stub.toml` and add its symbols to `PhpStormStubsMap.php` |

When syncing with upstream, compare symbols per file rather than text: most
text differences are intentional local fixes.

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

## Testing

Add a fixture under `crates/mir-analyzer/tests/fixtures/` that uses the
changed symbol; a fixture without issue annotations asserts the code is clean.
The format is described in the repository's
[CONTRIBUTING.md](../../../CONTRIBUTING.md#writing-a-fixture-test).

```sh
cargo test -p mir-analyzer --test fixtures
```

## Rebuilding

After changing `build.rs` or adding stub files, rebuild to re-embed:

```sh
cargo build -p mir-analyzer
```
