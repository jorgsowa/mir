# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1]

### Added

- `StubSlice::file` and `StubSlice::global_vars` fields so a slice can describe the source file it came from and the `@var`-annotated globals it declares.
- `CodebaseBuilder` and `codebase_from_parts` in `mir-codebase` — compose a finalized `Codebase` from per-file `StubSlice`s without mutating shared state during collection.
- `DefinitionCollector::new_for_slice` and `DefinitionCollector::collect_slice` — a pure-function entry point that returns a `StubSlice` instead of writing to a `Codebase`. Enables downstream consumers (e.g. salsa queries) to treat Pass 1 as a pure computation.

### Changed

- `DefinitionCollector` now builds a `StubSlice` internally; the existing `new` + `collect` API is preserved as a shim that injects the slice on completion.
- `Codebase::inject_stub_slice` now populates `symbol_to_file` and `global_vars` when the slice has a `file` set.

## [0.7.0] - 2026-04-21

### Added

- **PHP-first stub pipeline** — stubs are now authored as PHP source files under `stubs/{ext}/` with `stub.toml` manifests and transformed into Rust via the new `mir-stubs-gen` codegen tool, replacing the monolithic hand-written `stubs.rs`. (#243)
- **First-party stubs for 30 PHP extensions** — bundled stubs cover common extensions (curl, pdo, json, mbstring, etc.), loaded into the codebase at startup. (#246)
- **19 additional bundled-with-PHP extensions** — calendar, exif, ftp, gd, gettext, opcache, pgsql, phar, readline, shmop, soap, sqlite3, sysvmsg, sysvsem, sysvshm, tidy, xmlreader, xmlwriter, xsl. (#251)
- **`UndefinedConstant` issue** — the analyzer now emits `UndefinedConstant` for references to undefined global and class constants. (#242)
- **Target PHP version plumbed into `ProjectAnalyzer`** — the analyzer accepts a target PHP version to gate version-specific behavior. (#249)

### Changed

- Upgraded php-rs-parser and php-ast to 0.9; upgraded toml, quick-xml, and criterion to latest. (#245)

### Performance

- **BLAKE3 for cache hashing** — replaced SHA-256 with BLAKE3 for the incremental cache and deduplicated per-file hashing. (#244)

### Fixed

- **Leading backslash in `use` imports** — fully qualified use-imports (`use \Foo\Bar;`) now resolve correctly by stripping the leading backslash. (#247)
- **`composer.json` detection from path argument** — when invoked with a path argument, mir now walks up from that path to locate `composer.json` instead of only checking the CWD. (#247)

### CI

- Jobs are now gated (lint → stubs-up-to-date → test) and a dedicated step verifies that regenerated stubs match the committed generated files. (#250)

## [0.6.0] - 2026-04-19

### Added

- **Recurse into nested function and class bodies** — the analyzer now descends into nested function declarations and class definitions inside method/function bodies, catching issues in inner scopes that were previously invisible. (#223)
- **`UndefinedClass` for `extends`/`implements`** — emit `UndefinedClass` when a class extends or implements a type that does not exist in the codebase or stubs. (#224)
- **`InvalidScope` for `$this` in invalid context** — emit `InvalidScope` when `$this` is used outside of an object method (e.g., in a static method or free function). (#220)
- **Real-world Criterion benchmark suite** — added a benchmark that runs analysis over a realistic PHP codebase for continuous performance regression tracking. (#219)

### Fixed

- **Intersection type hints** — `type_from_hint` now correctly resolves intersection types (`A&B`), fixing false positives in type-narrowing and parameter checks. (#221)

## [0.5.2] - 2026-04-19

### Added

- **`StaticDynMethodCall` support** — dynamic static dispatch (`Foo::$method()`) is now handled as a distinct AST variant; evaluates arguments for taint propagation and returns `mixed`. (#216)

### Changed

- Upgraded php-rs-parser and php-ast to 0.8; migrated `FileParser` to `ParserContext` for O(1) arena reset on repeated parses. (#216)

### Performance

- **`MethodStorage` stored as `Arc`** — `own_methods` in all storage types now holds `Arc<MethodStorage>`, making method lookups an atomic refcount bump instead of a deep clone. (#213)
- **Skip re-analysis on unchanged content** — `re_analyze_file` returns cached results immediately when the file content hash matches, avoiding all four analysis phases on repeated LSP saves. (#204)
- **Skip `finalize()` on body-only changes** — `re_analyze_file` captures a structural snapshot before removal; if inheritance fields are unchanged after Pass 1, restores `all_parents` directly and skips the full class-hierarchy walk. (#205)

### Fixed

- **Trait-of-trait method resolution** — `get_method()` now walks the full transitive trait chain with a cycle guard, eliminating false `UnimplementedInterfaceMethod` errors for methods contributed by indirectly used traits. (#209)
- **`elseif` narrowing and branch merge** — elseif branches now correctly narrow on the parent `if` condition being false, and all elseif branches are folded into the post-if merge (previously only the last branch survived). (#211)
- **`TKeyedArray` foreach key type** — `infer_foreach_types` now derives `TLiteralString` / `TLiteralInt` keys from `ArrayKey` entries instead of always returning `TMixed`. (#211)
- **Switch fallthrough contexts** — non-diverging case contexts are now collected and merged into the post-switch type environment; chain-fallthrough into a diverging case is correctly propagated. (#212)

## [0.5.1] - 2026-04-18

### Performance

- **Reference index memory reduction** — intern reference keys with a lock-free `u32` interner, store all references in a flat `Vec<Ref>`, and compact into two CSR index arrays after Pass 2. Expected ~5× reduction in reference index memory. (#202)
- **Single-pass definition collection** — merged the pre-index and definition collection sub-passes into one parallel `par_iter`, eliminating the second parse of every file and removing the sequential serialisation barrier. (#196)

### Fixed

- Column offsets in diagnostics now use Unicode character counts consistently throughout mir-core. (#201)

## [0.5.0] - 2026-04-17

### Added

- **`issues_by_file()` on `AnalysisResult`** — group analysis issues by their source file path for easier per-file reporting. (#154)
- **Symbol reference location tracking** — `AnalysisResult::symbol_at` resolves the symbol under a given position, enabling LSP go-to-definition and find-references. (#185)
- **`ResolvedSymbol::file` and `codebase_key`** — extended resolved symbol information with the source file and codebase key for cross-file navigation. (#185)

### Changed

- Upgraded php-rs-parser and php-ast to 0.7. (#195)

### Fixed

- Property access symbols now use the identifier span and nullsafe accesses (`?->`) are tracked. (#189)
- Function, method, and static call symbols now use the identifier span rather than the full call expression span. (#192)
- `$this` is now injected into method context so `$this->method()` calls are correctly resolved by `symbol_at`. (#193)

## [0.4.1] - 2026-04-12

### Fixed

- **Diagnostic column offsets** — fixed `col_end` always being equal to `col_start` (resulting in zero-width diagnostic ranges) and column offsets being raw UTF-8 byte positions instead of character counts. Diagnostics now correctly highlight the full variable/expression range with proper multi-byte character handling. (#182)

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
