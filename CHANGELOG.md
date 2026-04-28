# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.16.0] - 2026-04-28

### Added

- Cross-file inferred return types (G6): a type-inference priming pass now runs all function and method bodies in parallel before the issue-emitting Pass 2, writing `inferred_return_type` for every symbol without recording reference locations. Callers no longer see `mixed` for callees whose Pass 2 had not yet completed. Covers the common depth-1 case; depth-N chains are addressed by Phase 4 (Salsa).
- Per-class `OnceLock` finalization (Phase 3 item 6): `ensure_finalized(fqcn)` lazily computes and memoizes each class's ancestor chain on first access via `DashMap<Arc<str>, OnceLock<Arc<[Arc<str>]>>>` with thread-local cycle detection. `finalize()` is now a warm-all wrapper; `remove_file_definitions()` evicts only the affected entries granularly.

### Performance

- Lazy finalization removes the pass barrier (Phase 3 item 7): the eager `finalize()` barrier that blocked all of Pass 2 until every ancestor chain was warm is removed. `ensure_finalized()` is now called at each `all_parents` read site (`get_method_inner`, `get_property_inner`, `get_class_constant`, `extends_or_implements`, `has_unknown_ancestor`, `collect_members_for_fqcn`, `ClassAnalyzer::analyze_all`, `check_trait_constraints`, `argument_type_satisfies_param`). Phase 3 is now complete.

### Fixed

- LSP incremental re-analysis: classes defined in an analyzed file but never referenced during Pass 2 had empty `all_parents` at snapshot time, causing `restore_all_parents` to silently restore empty ancestor chains on the LSP fast path. `file_structural_snapshot` now calls `ensure_finalized` for each symbol before capturing it.

## [0.15.0] - 2026-04-28

### Added

- Return type covariance for named-object overrides: `ClassAnalyzer` now delegates to `named_object_return_compatible` when checking overriding methods, catching cases where a child class returns an unrelated type instead of the declared parent return type. Mixed scalar+object unions still skip the check to avoid false positives.
- Type narrowing after `instanceof $this`: when the right-hand side of `instanceof` is `$this`, it is resolved to the current class FQCN before narrowing, eliminating false-positive `MixedMethodCall` and `UndefinedProperty` diagnostics on `if (!$other instanceof $this)` guards. (#144)

### Changed

- `stmt.rs` split into `stmt/` sub-module (`mod.rs`, `loops.rs`, `return_type.rs`), following the same pattern as `call/`. No behavior change.

## [0.14.0] - 2026-04-28

### Added

- Generic template substitution extended to array shapes (`TKeyedArray`, `TNonEmptyArray`, `TNonEmptyList`), callable/closure types, conditional types, and intersection types. Variable calls (`$fn()`) on `TClosure`/`TCallable` now resolve the correct return type instead of `mixed`. `TIntersection` method calls resolve against the part that owns the method. Docblock parser gains `array{key: T}` shape syntax and `callable(T): R` / `Closure(T): R` parsing.
- `ParsedDocblock::is_inherit_doc` flag: set when `@inheritDoc`, `@inheritdoc`, or `{@inheritDoc}` is present in a docblock, enabling LSP clients to walk the inheritance chain for hover and completion without implementing resolution in mir itself.

### Fixed

- LSP / incremental re-analysis: `inject_stub_slice` now populates `file_namespaces` and `file_imports` in the codebase, fixing false-positive `UndefinedClass` diagnostics for `use`-aliased classes after any incremental re-analysis triggered by `re_analyze_file`.

### Changed

- `Location` type unified in `mir-types`; internal codebase storage switched from byte offsets to `(line, col_start, col_end)`. All `mark_*_referenced_at()` methods now accept line/column instead of byte offsets. Columns use 0-based Unicode code-point counts (LSP UTF-32 encoding); UTF-16 conversion happens at the LSP boundary for clients that do not advertise UTF-32 support. Existing on-disk caches silently rebuild on the next run.

### CI

- Docs deploy now invokes a reusable `workflow_call` path to `docs.yml` so the deployment runs under a branch-authorized context instead of directly from a tag, fixing GitHub Pages environment protection failures.

## [0.13.0] - 2026-04-28

### Added

- Interactive WASM playground embedded in the docs site: select PHP version (8.1–8.5), type PHP code, and see live diagnostics with underline overlays and severity-colored cards. (#287)

### Changed

- Docs site logo added to README and top bar; branding updated.
- php-ast and php-rs-parser bumped to 0.9.6.

### CI

- Node.js version in docs deploy workflows raised from 20 to 22 (Astro now requires >=22.12.0).

## [0.12.0] - 2026-04-27

### Added

- `PossiblyInvalidArgument` issue: emitted when a `false|T` union value is passed to a parameter that does not accept `false`, surfacing potential type mismatches that were previously silently widened to `mixed`.
- Backed enum `->value` and `->name` access now returns a precise inferred type (`TLiteralString` / `TLiteralInt` for `->value`, `TLiteralString` for `->name`) instead of `mixed`.
- `call_user_func` and `call_user_func_array` string callables (e.g. `'ClassName::methodName'`) are now tracked as real call references, fixing false-positive stub warnings on those forms.

### Fixed

- Infinite recursion on circular `@mixin` references: the mixin resolver now carries a seen-set and breaks cycles instead of stack-overflowing.
- Benchmark harness: rayon stack size raised to 16 MiB and the global thread pool is initialised explicitly, preventing stack overflows on deeply recursive PHP files during benchmarking.

### CI

- `timeout-minutes` added to all workflow jobs and a concurrency group added to the CI workflow to cancel superseded runs.

## [0.11.1] - 2026-04-26

### Fixed

- Release CI: GitHub Release is now created from the CHANGELOG before binaries are uploaded, fixing a race condition where `upload-rust-binary-action` failed with "release not found".

## [0.11.0] - 2026-04-26

### Added

- `InvalidDocblock` issue: emitted when a type annotation in a docblock cannot be parsed (malformed syntax). (#282)
- Injectable user stubs: `<stubs><file name="..."/>` and `<stubs><directory name="..."/>` elements in `mir.xml` / `psalm.xml` load additional stub paths before analysis; stub files are not themselves analyzed for errors. (#285)
- `phpVersion` can now be set as an XML attribute on the root `<mir>` or `<psalm>` element (e.g. `<mir phpVersion="8.2">`), matching Psalm's config syntax, in addition to the existing child-element form. (#285)

### Changed

- phpstorm-stubs is now vendored directly in `stubs/` (tracked in git) instead of a git submodule. External contributors no longer need to run `git submodule update --init`. (#283)
- Documentation site migrated from mdBook to Astro Starlight; issue-kind reference pages are now split into individual pages grouped by category.

## [0.10.0] - 2026-04-26

### Added

- Composer package `miropen/mir-php`. A `post-install-cmd` / `post-update-cmd` hook downloads the prebuilt `mir` binary matching the installed version and host platform from GitHub Releases, verifies the SHA-256 sidecar, and exposes `vendor/bin/mir`. Single-entry extraction with strict path-traversal and symlink rejection. Supported targets: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`.
- `Release` GitHub Actions workflow building and uploading per-target archives + sha256 sidecars on `v*` tags.
- `NullArgument` issue: emitted when a literal `null` is passed to a non-nullable parameter (previously subsumed by `InvalidArgument`). Severity: warning.
- `UnusedFunction` issue: emitted for free functions that are never called when `find_dead_code` is enabled.
- `InvalidPropertyAssignment` issue: emitted when a value of an incompatible type is assigned to a typed property. Handles class inheritance via the codebase.

### Fixed

- `cargo install mir-cli` references in README and docs corrected to `mir-php` (the actual crate name).
- Panic in docblock extraction when source text before a declaration contains multibyte characters (e.g., `→`). `find_preceding_docblock` now correctly advances past multibyte chars when scanning for word boundaries.

## [0.9.1] - 2026-04-26

### Added

- `Location.line_end` field — all issues now carry an end line number, enabling multi-line range highlighting in editors and code scanning tools. (#270)
- SARIF output: `region.endLine` populated from `line_end`. (#270)
- SARIF output: results now include `rank` (Error → 90, Warning → 95, Info → 99) matching Psalm's scoring range. (#270)
- SARIF output: rules now include `properties.tags` (`"security"` for taint issues, `"maintainability"` for all others). (#270)
- Psalm docblock parity: `@psalm-assert-if-false` type narrowing. (#267)
- Psalm docblock parity: `@psalm-import-type` type alias imports. (#267)
- Psalm docblock parity: `@psalm-param` and `@psalm-return` type narrowing annotations. (#267)

### Fixed

- SARIF output: `startColumn`/`endColumn` are now correctly 1-based per SARIF 2.1.0 §3.30.5 (previously off by one). (#270)
- SARIF output: rules now include `defaultConfiguration.level` so the GitHub Code Scanning rules panel shows severity. (#270)
- SARIF output: results now include `partialFingerprints.primaryLocationLineHash` (FNV-1a of rule name + snippet) so GitHub Code Scanning can track findings across commits. (#270)
- Static calls now correctly check for `__callStatic` (not `__call`) when suppressing `UndefinedMethod` on missing static methods. (#271)
- Magic method dead-code exclusion now uses lowercase keys matching `own_methods` storage, so `__callStatic`, `__toString`, and `__debugInfo` are correctly exempted from `UnusedMethod` reports. (#271)
- `__unserialize` added to `MAGIC_METHODS_WITH_RUNTIME_PARAMS`, preventing its `$data` parameter from being flagged as unused. (#271)
- Trait docblock parsing now falls back to raw-source lookup when php-rs-parser absorbs the trait-level docblock, ensuring `@psalm-require-extends` and `@psalm-require-implements` are correctly detected. (#267)

### Changed

- Bumped blake3, php-ast, php-lexer, and php-rs-parser to latest. (#272)

## [0.9.0] - 2026-04-26

### Added

- Trait method bodies are now analyzed in Pass 2; diagnostics (`UndefinedFunction`, `UndefinedMethod`, unused variables, etc.) are emitted for code inside traits. (#264)
- `UnreachableCode` issue — statements following a terminator (`return`, `throw`, `exit`, `die`) in the same block are now flagged; nested closures are analyzed with a fresh context and are not affected by divergence in the outer block. (#262)

### Fixed

- `PossiblyUndefinedVariable` promoted to `Warning` severity, making it visible at the default error level and matching Psalm's behavior. (#261)
- 10 false-positive `UndefinedMethod` reports eliminated: dynamic method calls via variable expressions (`$obj->{$var}()`) no longer trigger a spurious lookup, and private trait methods are now correctly accessible from classes that use the trait. (#260)
- Improved Psalm docblock parity. (#265, #266)

## [0.8.0] - 2026-04-25

### Added

- `PhpVersion::LATEST` constant (currently `8.5`) — used as the default when no explicit version is configured.
- `ProjectAnalyzer::with_php_version` builder method to set the target PHP version.
- `@deprecated` tag messages are now included in `Deprecated` issue descriptions.
- `php_version` is now propagated through `StatementsAnalyzer` and `ExpressionAnalyzer` for version-gated checks.

### Fixed

- `UndefinedClass` is now detected in 7 previously-silent code paths.
- Static method call spans now use the parser span for the method name rather than manual offset arithmetic.
- Windows build: `canonicalize()` returns `\\?\`-prefixed UNC paths on Windows; the build script now strips that prefix before embedding stub paths in `include_str!`.

### Changed

- `ProjectAnalyzer::php_version` field is now `Option<PhpVersion>` (`None` = use `PhpVersion::LATEST`); previously it was a bare `PhpVersion` defaulting to 8.4.
- Bumped `php-rs-parser`, `php-ast`, and `php-lexer` to 0.9.2.

### Performance

- `IssueBuffer::add` deduplication changed from an O(n) scan to a `HashSet` lookup.

## [0.7.3] - 2026-04-25

### Added

- Cross-file `.phpt` fixture format with `===file:Name.php===` sections and optional `composer.json` for PSR-4 lazy-loading scenarios; 21 new cross-file fixtures added.
- `===config===` section in `.phpt` fixtures for per-fixture settings (`php_version`, `find_dead_code`); dead-code fixtures now declare this in config instead of relying on a hard-coded category list.
- New `stub_behavior/` fixtures covering `stdClass`, `preg_match`, `sscanf`, `array_map` null callback, and `array_keys` optional filter.
- Correctness tests for `inject_stub_slice` covering symbol overwrite, `symbol_to_file` updates, `global_vars` cleanup on `remove_file_definitions`, and `StubVfs` roundtrip navigability.

### Changed

- Switched stubs from generated Rust files (`mir-stubs-gen`) to phpstorm-stubs loaded at build time via `CUSTOM_STUB_FILES`; the `mir-stubs-gen` crate is removed.
- Unified single-file and multi-file `.phpt` fixture parsers into a single `parse_phpt` function; existing `===source===` markers renamed to `===file===`.

### Fixed

- `UnimplementedAbstractMethod` and `UnimplementedInterfaceMethod` errors now report the method name with its original declared casing instead of the lowercase-normalized form.

## [0.7.2] - 2026-04-24

### Changed

- Bumped `php-rs-parser`, `php-ast`, and `php-lexer` to 0.9.1.

## [0.7.1] - 2026-04-22

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
