# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-04-12

### Added

- **JetBrains phpstorm-stubs integration** — mir now uses the authoritative [phpstorm-stubs](https://github.com/JetBrains/phpstorm-stubs) repository as the source for PHP built-in definitions. This provides comprehensive coverage of 500+ functions, 100+ classes, and 200+ constants across 33 PHP extensions. (#181)
- **Global variable registry** — new `@var` annotation support for tracking globally-scoped variables declared outside of function/class scope. Reduces false positives in `UndefinedVariable` checks. (#160)

### Changed

- **Dependency updates** — upgraded php-rs-parser and php-ast to v0.6.0 for improved parsing robustness and performance.

### Fixed

- `is_builtin_function` now uses the full loaded stubs to properly detect built-in functions across all extensions.

## [0.3.0] - 2026-04-10

### Added

- **Generic type covariance and contravariance** — full support for `@template` type parameter variance annotations in classes and methods. (#109)
- **Circular inheritance detection** — emit `CircularInheritance` error when classes form circular inheritance chains. (#110)
- **Test fixture infrastructure** — 22 new test fixtures covering previously uncovered rule categories, bringing fixture test count to 119. (#98)

### Changed

- **AST doc_comment refactor** — switched from manual docblock discovery to using AST `doc_comment` fields for more reliable comment association. (#107)
- Removed `mir-test-utils` crate to eliminate circular dependency structure. (#106)

### Fixed

- **Class-level issue reporting** — proper source locations (line/column in `storage::Location`) and code snippets now emit correctly for class-level issues. (#105)
- **Magic method parameters** — `UnusedParam` checks now exclude magic method parameters (`__construct`, `__get`, etc.). (#108)

## [0.2.1] - 2026-04-09

### Changed

- Upgraded php-ast and php-rs-parser to v0.5.0.

### Fixed

- Proper source mapping threading from `ParseResult` through the analysis pipeline.

## [0.2.0] - 2026-04-08

### Added

- **SymbolTable adoption** — parallel pre-indexing of file imports, namespaces, and known symbols for better scalability.
- **SourceMap and CommentMap** — adopted from php-ast for reliable line/column resolution and comment association.
- Test fixture infrastructure with 96 fixture-based tests across 10 rule categories.

### Fixed

- Reduced `UnusedVariable` false positives from 405 to 127 through improved read tracking in closures and assignment contexts.

## [0.1.0] - 2026-03-15

### Added

- Initial release of mir, a fast incremental PHP static analyzer written in Rust.
- Core features: type system, type inference, call checking, class analysis, dead code detection, taint analysis, incremental caching, parallel analysis.
- Comprehensive built-in PHP function and class coverage.
