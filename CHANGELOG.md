# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.42.0] - 2026-06-15

### Added

- `AnalysisSession::class_imports(file)` → `Vec<(alias, fqcn)>` — returns the use-import alias map for a file as `(short_name, fully_qualified_name)` pairs. Completion handlers can use this to expand a short class name written before `::` into its FQN before looking up static members, mirroring the alias expansion already performed by `symbol_at` + `definition_of`.
- Vendor `autoload.files` globals (e.g. Laravel helper functions) are now lazy-loaded automatically on first analysis. Previously callers had to invoke a manual eager-index step; any consumer that omitted it received false-positive `UndefinedFunction` diagnostics for every call to those globals.

### Fixed

- Diagnostic column numbers are now 0-based throughout, matching the LSP UTF-32 convention documented in `mir_types::Location`. Body-analysis diagnostics were previously emitting 1-indexed columns, inconsistent with collector-stored diagnostics (which were already 0-indexed).
- Classes referenced only in docblock annotations (`@param`, `@return`, `@var`, `@extends`, `@implements`) are now pre-loaded during AST prioritization. Previously such classes were invisible to the pre-loader; method and property checks on the annotated variable would silently degrade to `mixed` when the class had not yet been eagerly indexed.

## [0.41.0] - 2026-06-15

### Added

- `IfThisIsMismatch` (MIR0902) — emitted when a method's `@if-this-is` type constraint is violated at a call site. Template-aware constraint checking enables precise type narrowing for receiver type refinements.
- `DocblockTypeContradiction` (MIR0406) — emitted when a comparison operator (`===`, `<`, `<=`, `>`, `>=`) is used with values that cannot satisfy the condition given their inferred types. Detects impossible assertions and dead code in conditionals.
- `UnevaluatedCode` (MIR0407) — emitted when a `switch`/`match` statement on `gettype($x)` contains arms that `gettype()` never returns (e.g., `"int"` when the actual return is `"integer"`), or when the argument's inferred type cannot produce those values.
- `MixedReturnStatement` (MIR1212) — emitted when a function with a declared non-void return type returns a `mixed` value (e.g., `array_pop()` in a `string`-returning function).
- Integer range types now tracked for `count()`/`sizeof()` → `int<0, max>` (or `int<1, max>` for non-empty), `strlen()`/`mb_strlen()` → `int<0, max>`, and arithmetic operations preserve range bounds. Comparison-driven narrowing (e.g., `if ($i < count($a))`) now refines loop variables to their valid index ranges.
- `array_map()` and `array_filter()` now infer precise result element types from their callbacks instead of returning bare `array`.
- Vendored Redis and Memcached phpstorm-stubs extension directories, fixing ~1,400 `UndefinedClass` false positives on Laravel codebases that use these PECL extensions.
- phpstorm-stubs `#[LanguageLevelTypeAware]` and `#[PhpStormStubsElementAvailable]` attributes are now resolved against the configured target PHP version. This honors ~2,400 previously-dropped declaration sites and eliminates spurious `|false` returns for version-specific function signatures (e.g., `explode()`/`pack()` on PHP 8.x).
- Filesystem and unserialize taint sinks: `file_get_contents()`, `file_put_contents()`, `unserialize()` and related functions now propagate taint in `TaintedFilesystem` and `TaintedUnserialization` issue kinds.
- Symbol reference recording for static-call class name tokens, enabling go-to-definition and find-references on `ClassName::method()` expressions.

### Fixed

- Class-level template parameters are now correctly resolved in method parameter types during generic method binding.
- Named-object arguments now satisfy bare `object` parameter types via `named_object_subtype` checking.
- `[object, "method"]` array literals are now recognized as valid callables, fixing false `InvalidArgument` diagnostics.
- Docblock-only properties (declared via `@property` annotations) are now correctly typed as nullable and not flagged as uninitialized.
- Property `@var` docblock annotations now resolve class-level template parameters, enabling precise typing for generic class properties.
- Surplus arguments to closure calls no longer emit false `TooManyArguments` diagnostics when the closure arity is unknown.
- Array-access narrowing: `isset($arr[$key])` and `??` operators now narrow both the array base and key existence.
- `is_object()` type guards now correctly narrow `mixed` to `object` in conditional branches.
- `unset($arr[$key])` now counts as a read of the variable, fixing false `UnusedVariable` diagnostics.
- Static property reads (`self::$prop`) now correctly count as property uses.
- Bare `return;` statements are now valid in functions with nullable or void-union return types.
- `defined()` and `function_exists()` guards now narrow constant/function references in conditional branches.
- By-reference closure captures (`use (&$var)`) now auto-create the captured variable if it doesn't exist.
- Variable assignments inside `match` arm conditions now correctly define the variable for use in the arm body.
- `self`, `static`, `parent`, and `$this` resolution in trait bodies now correctly targets the consuming class instead of the trait.
- `new static` is now allowed in abstract classes, delegating to concrete subclasses at runtime.
- `stdClass` now permits dynamic property access and assignment without emitting `UndefinedProperty` diagnostics.
- Fully-qualified attribute names in `#[...]` are now honored in attribute resolution.
- `Numeric` and `Resource` are no longer treated as reserved class names in the parser.
- `--clear-cache` now correctly targets the project-local cache directory (`.mir/cache`) instead of always searching the platform default cache dir.
- Result cache now invalidates when the running binary, target PHP version, or user-configured stubs change.
- Wrong-case checks now extend to full FQCN namespace segments, catching case mismatches in any part of the class name.

### Changed

- `mir-analyzer` module structure refactored for maintainability: `batch.rs`, `class.rs`, `parser/docblock.rs`, `session.rs`, and `body_analysis.rs` split into dedicated submodules.
- PHP parser suite (`php-rs-parser`, `php-ast`, `php-lexer`, `phpdoc-parser`) upgraded to 0.18.0 for improved parsing robustness.

### Performance

- Large false-positive reduction on the Laravel reference corpus: the vendored Redis/Memcached stubs and version attributes support reduce `UndefinedClass` from 617 to 114 (an 82% reduction on the reference benchmark).

## [0.40.0] - 2026-06-13

### Added

- `TypeDoesNotContainType` — impossible `switch` case values (literal cannot intersect the switch subject type) and impossible `match` arm conditions (same scalar/literal intersection check) are now reported. `MixedAssignment` is now also emitted when a `foreach` value variable is bound from a mixed-typed iterable (previously this path bypassed the mixed check).
- Purity enforcement: `ImpurePropertyAssignment` (MIR1700), `ImpureMethodCall` (MIR1701), `ImpureGlobalVariable` (MIR1702), `ImpureStaticVariable` (MIR1703) — emitted when a `@pure`/`@psalm-mutation-free`-annotated function mutates a parameter's property, calls an impure method on a parameter, or accesses a global/static variable.
- `ImpureFunctionCall` (MIR1704, Warning) — emitted when a `@pure`-annotated function calls a named function not itself marked `@pure`.
- `UnusedClass` (MIR0507, Info) — `final` class declared but never directly referenced. Restricted to `final` classes to avoid false positives from subclassing or type-hint uses.
- `ArgumentTypeCoercion` (MIR0225, Info) — emitted when an argument is a supertype (parent class) of the expected parameter type. Previously these calls were silently accepted.
- `PropertyTypeCoercion` (MIR0226, Info) — emitted when a property assignment uses a supertype of the declared property type. Previously emitted as the higher-severity `InvalidPropertyAssignment`; correctly distinguished as a lower-severity coercion case.
- `TaintedLlmPrompt` (MIR0804, Error) — emitted when a value derived from tainted user input reaches a parameter annotated with `@taint-sink llm_prompt`. Parser now recognises `@taint-sink kind $param` docblock tags; sink params are stored on `FunctionDef`/`MethodDef`.
- `UnusedPsalmSuppress` (MIR0508, Info) — emitted when a `@psalm-suppress`, `@suppress`, or `@mir-suppress` annotation does not match any actual issue in the analysed file. Self-suppression (`@suppress UnusedPsalmSuppress`) silences its own warning.
- `UnsupportedReferenceUsage` (MIR1506, Warning) — emitted when a PHP reference assignment (`$b = &$x`) is used.
- `NoInterfaceProperties` (MIR1504, Info) — emitted when a property is read or written on an interface annotated with `@seal-properties`/`@psalm-seal-properties` but not declared via `@property`/`@property-read`/`@property-write`.
- `MissingConstructor` (MIR1507, Info) — emitted when a concrete class has at least one non-nullable uninitialized property anywhere in its ancestor chain but defines no constructor.
- `MixedFunctionCall` (MIR1211, Info) — emitted when a variable of mixed type is invoked as a function via a dynamic call expression.
- `MissingClosureReturnType` (MIR1105, Info) — emitted when a closure has no native return type and no preceding `@return` docblock (Full mode only).
- `InvalidArrayOffset` (MIR0300, Error) — emitted when an object, array, or closure is used as an array subscript key (types PHP cannot coerce to a valid array key).
- `PossiblyInvalidArrayAccess` (MIR0227, Info) — emitted when a union type contains some members that support `[]` and some that do not.
- `DeprecatedMethod` (MIR1002) — instance deprecated method calls now emit `DeprecatedMethod` instead of `DeprecatedMethodCall`, reserving `DeprecatedMethodCall` for static calls and `__clone` dispatch.
- `MixedReturnStatement` (MIR1212, Info) — emitted when a function with a declared non-void, non-mixed return type returns a value that infers to mixed (e.g. `array_pop()` returned from a `string` function).
- phpstorm-stubs `#[LanguageLevelTypeAware]` and `#[PhpStormStubsElementAvailable]` attributes are now resolved against the configured target PHP version. This correctly models ~2,400 previously-dropped declaration sites across the stub corpus and eliminates spurious `|false` returns for `explode()`/`pack()` on PHP 8.x (those functions throw rather than return false since 8.0).
- Vendored Redis and Memcached phpstorm-stubs extension directories. These PECL extensions were previously excluded and had no fallback resolution path after the submodule loader was removed, causing false `UndefinedClass` for `Redis`/`Memcached` in Laravel codebases.

### Fixed

- Result cache now invalidates when the running binary, target PHP version, or user-configured stubs change. Previously the cache keyed validity on file content hash only, leaving unchanged files serving stale diagnostics after a version upgrade, `--php-version` change, or stub set update.
- `--clear-cache` now correctly targets the project-local cache directory (`{composer_root}/.mir/cache`) instead of always looking at the platform default cache dir and attempting to remove a `cache.json` that no longer exists (the format is `cache.bin`), making it a functional operation for normal project runs.

## [0.39.0] - 2026-06-12

### Added

- `UnnecessaryVarAnnotation` (Info) — a `@var` annotation on a simple assignment whose declared type exactly matches the inferred type is flagged as redundant. The comparison is exact, with no literal widening: `@var string` on `$s = 'hello'` changes the type (literal → base) and is therefore not reported. Narrowing annotations, mixed-typed right-hand sides, and non-assignment statements stay silent.
- `MismatchingDocblockReturnType` / `MismatchingDocblockParamType` (Info) — a `@return`/`@param` docblock that contradicts the native type hint on a top-level function is now reported. Refinements never fire: the comparison uses PHP type families (with `int → float` coercion and `callable`'s string/array/object forms modeled), so e.g. `literal-string`/`non-empty-list<…>` against their base hints stay silent. Object-vs-object falls back to an inheritance-aware subtype check when every named class is known; unresolved names (templates, `::class` refs, unmodeled refinement syntax) stay silent.
- `MissingReturnType` / `MissingParamType` (Info) — top-level functions with neither a native hint nor a docblock type are now reported (previously only interface methods were checked), on all three analysis paths (per-scope salsa, batch typed, pure per-function).

### Fixed

- `reanalyze_dependents` no longer deadlocks on workspaces with high dependent fan-out. The per-dependent warm-up (`prepare_ast_for_analysis`, introduced in 0.37.0) loads classes by mutating shared salsa inputs, and salsa input mutation blocks until every other database handle is released. Running the warm-up inside the parallel rayon worker meant a worker mutated the storage while sibling workers held live snapshots mid-`analyze_file`, so the write blocked on them forever — hanging indefinitely on high-fan-out workspaces. Warm-up now runs before the parallel read-only analyze loop, with each iteration holding only a scoped snapshot that is dropped before any input write, restoring the "no input writes while a snapshot is live" invariant. Covered by a regression test (`reanalyze_dependents_lazy_load_warmup_does_not_deadlock`).
- Large false-positive reduction on the Laravel reference corpus across several diagnostic kinds (each fix ships with regression fixtures, including the negative cases):
  - **Template binding**: trailing variadic params now bind every remaining argument (unwrapping `array<X>` docblock types to their element type per argument); a `class-string<T>` union alternative consumes class-string arguments so a sibling bare `T` no longer absorbs them; method-level `@template` shadows a same-named class template during argument checking; and a docblock description following a `@template` line is no longer misparsed as a bound. Removes ~1350 FPs (6148 → 4781), dominated by Mockery intersection mocks.
  - **`InvalidStringClass`**: `new $x` where `$x` is `mixed` or a template param no longer fires — `mixed` is a `Mixed*` concern, and a template bound may be a class-string. Removes 501 FPs (4781 → 4280).
  - **`UndefinedMethod`**: `$this->m()` / `static::m()` inside a trait body is suppressed (the consuming class may provide the method, so traits join interfaces/abstract classes), and inaccessible protected/private calls dispatched through `__call` (e.g. `Macroable`, Mockery partial mocks) no longer error. 756 → 168.
  - **`UnusedVariable` / `UnusedForeachValue`**: path-accurate liveness — closure `use()` captures and closure/arrow-body reads consume the outer write; branch merges no longer resurrect a write consumed on one path; multiple pending write locations per variable are tracked (pre-loop and loop-body writes); and `switch` with a `default` arm no longer merges the impossible no-match path. 938 → 376 and 106 → 54.
  - **`UnusedVariable`**: an assignment in argument position (`f($x = expr)`, `->andReturn($mock = m::mock(...))`) now counts as a use across all call shapes (function, dynamic-callee, method, static, `new`). 376 → 243.
  - **`UndefinedFunction` / `InvalidTemplateParam`**: a string passed to a union param with non-callable alternatives is no longer validated as a function name (157 → 3); template bounds are not checked against bindings that still contain unresolved placeholders (`self`/`static`/`parent`, template params) (103 → 42); and `class-string<T>` binding coerces class-name-shaped string literals such as `m::mock('Foo\Bar')` without `::class`.
  - **`InvalidArgument`**: an array passed to a `callable|array|null` param matches the array alternative instead of being forced into the `[object, "method"]` callable shape. Removes 169 FPs (618 → 449).

## [0.38.0] - 2026-06-12

### Added

- `IssueKind::default_severity_for_code` — reverse lookup from a stable error code (e.g. `"MIR0005"`) to its default severity, for callers holding a bare code string (config files, suppression annotations, serialised diagnostics).

### Changed

- Property-access and method-call symbol recording now reuses the declaring class from type resolution instead of re-walking the inheritance chain, removing a redundant ancestor-chain walk per property access and per method call.

### Fixed

- Closure and arrow-function parameter/return type hints (`function (Foo $x) {}`, `fn (Foo $x) => ...`) now contribute reference-index entries and `ClassReference` symbols, so find-references and hover cover closure usages.
- `$x instanceof Foo` now records a `ClassReference` symbol at the class-name span, unblocking hover and the `symbol_at` → `references_to` round-trip for instanceof sites.
- Property references and symbols now key on the declaring class (as `find_property_in_chain` returns it) instead of the receiver type, fixing find-references and `symbol_at` for inherited properties accessed through a subtype.

## [0.37.0] - 2026-06-11

### Added

- Per-scope tracked inference queries (`file_scopes`, `infer_scope`) for granular type inference memoization at function/class declaration and file-frame scope levels.
- Batch-mode symbol collection opt-out via `BatchOptions::skip_symbols` for performance optimization in batch analysis runs.

### Changed

- `analyze_file` now assembles results from per-scope memos instead of a single whole-file analysis walk, improving incremental re-analysis efficiency.
- Reference locations architecture refactored: `RefIndex` consolidates three independent reference maps (`reference_locations`, `file_references`, `symbol_referencers`) into a single tracked structure.
- Dependent re-analysis now drives through the `analyze_file` query for salsa-validated memoization, replacing per-file re-parsing and full re-analysis on every edit.
- Reverse dependencies now derived from a tracked query (`file_structural_deps`) instead of an in-memory map, improving incremental consistency.

### Fixed

- Reference-location synchronization drift eliminated by consolidating three independent maps into `RefIndex`.

## [0.36.0] - 2026-06-11

### Added

- `MissingReturnType` (MIR1201) and `MissingParamType` (MIR1200) — emitted for interface methods that lack `@return` or `@param` docblock annotations when not otherwise typed.
- `MixedArgument` (MIR0221) and `MixedAssignment` (MIR0222) — emitted when a `mixed`-typed value is passed to a parameter expecting a concrete type, or assigned to a typed property.
- `MixedArrayAccess` (MIR0223), `MixedArrayOffset` (MIR0224), `MixedPropertyFetch` (MIR0225), and `MixedPropertyAssignment` (MIR0226) — emitted when `mixed` is used in array/property access contexts.
- `MissingPropertyType` (MIR1202) — emitted for untyped class and trait properties when `find_dead_code` is enabled.
- `ForbiddenCode` (MIR1301) — detects code marked with `#[Forbidden]` attribute; use `#[Forbidden("reason")]` on methods/functions to flag uses as errors.
- `@trace` docblock annotation — mark variables and expressions with `/** @trace $var */` to emit an `@trace` informational diagnostic, aiding debugging without leaving analyzer artifacts in code. Useful for development and CI integration.
- `PossiblyInvalidArgument` (MIR0205) — enhanced to flag partial type-union overlaps, not just complete mismatches. Emitted when a union contains only some valid argument types.
- Type-checking for `TClosure` and `__invoke` method calls: generic template parameters are now resolved at call sites, enabling precise type narrowing on closure return values.
- `@no-named-arguments` enforcement: methods/functions marked with this attribute now emit `InvalidArgument` when invoked with named arguments.
- Duplicate declaration detection: `DuplicateClass`, `DuplicateInterface`, `DuplicateTrait`, `DuplicateFunction`, and `DuplicateConstant` now detect and report redeclarations across the entire codebase.
- Psalm compatibility: all 1843 fixture tests now pass, including un-ignoring 120+ Psalm-specific test cases covering edge-case behaviors.

### Fixed

- Constructor-promoted property handling: `UnusedParam` and `UnusedVariable` false positives eliminated for promoted properties accessed through property-assignment or constructor side effects.
- `if`-condition variable assignment detection: variables assigned in `if` condition expressions (e.g., `if ($x = foo())`) are no longer incorrectly flagged as unused.
- Negated `instanceof` guard narrowing: type refinement now correctly applies at receiver position (`$obj instanceof X` and `!$other instanceof $this`).
- User-defined stub registration now uses Salsa `Durability::HIGH`, improving incremental re-analysis performance when stubs are unchanged.
- Readonly promoted properties and compound-assignment edge cases in destructuring contexts.
- Operand and iteration gaps now match Psalm parity across type-checking and narrowing behaviors.
- `TLiteralString` subtype narrowing: numeric literal strings now correctly match `TNumericString` bounds.
- Globally-qualified type hints (`\Closure`, `\Generator`, etc.) in namespaced files now resolve correctly without prepending the current namespace.
- Generator `bare return;` statements no longer emit false `InvalidReturnType` diagnostics.
- `try`-body divergence is now preserved when all `catch` blocks also diverge, preventing unreachable-code false positives.
- `ImplicitToStringCast` suppression for classes implementing `\Stringable` and when argument union contains non-string arms.
- `@param` docblock generic type hints now take precedence over plain array hints for promoted properties.

### Changed

- All 1843 fixture tests now pass without ignores, improving test coverage visibility and closing known gaps in Psalm parity.

## [0.35.1] - 2026-06-10

### Fixed

- `DuplicateClass` no longer fires when two classes share the same name in separate unbraced namespace blocks.
- `abs(int)` now returns `int` instead of `float|int`.
- Symbol lookup now records parameter declaration sites as `Variable` symbols, enabling go-to-definition on function/method parameters.
- Symbol lookup now resolves gap cursors in method chains via `expr_span` fallback, fixing missed definitions in chained calls.

### Changed

- PHP parser and phpdoc-parser updated to 0.17.0.

## [0.35.0] - 2026-06-09

### Added

- `UnhandledMatchCondition` — emitted when a `match` expression is non-exhaustive: empty match (no arms), string literal union subject with uncovered values, or pure (non-backed) enum subject with missing cases. Enum method bodies are now included in the body-analysis pipeline, enabling exhaustiveness detection inside enum methods.
- `AbstractMethodCall` now fires when an abstract static method is called by explicit class name (e.g. `Base::bar()` where `bar()` is abstract). Self/static/parent calls remain exempt.
- `InvalidDocblock` now covers three additional categories: `int<min,max>` ranges with invalid boundaries or wrong ordering; `array<K,V>` with a key type that is not a subtype of `int|string`; and `@method` annotations that are empty, contain invalid characters, or declare by-reference parameters.
- `InvalidDocblock` is now also emitted for `@template` annotations on closure and arrow-function expressions, where they have no effect.
- Trait method signatures are now checked against interface requirements: when a class implements an interface via `use T`, the trait method's signature is compared against the interface declaration and `MethodSignatureMismatch` is emitted for incompatible signatures.
- Trait `insteadof` conflict resolution is now applied during method lookup (go-to-definition and call resolution resolve to the winning trait instead of whichever was indexed first).

### Fixed

- `__get` return type is now propagated to magic property-access inference: accesses that fall through to `__get` carry the declared return type instead of always resolving to `mixed`.
- `enum::cases()` now synthesizes `list<EnumType>` instead of `mixed`, allowing `foreach` loop variables to be typed as the specific enum and enabling `UnhandledMatchCondition` to fire on enum matches.
- `SourceFile` text is now freed on removal: the `Arc<str>` content is nulled immediately after workspace index cleanup, releasing file content memory that was previously retained indefinitely due to Salsa 0.27 lacking a delete API.
- Salsa LRU cap added to `collect_file_declarations` (`lru = 4096`), matching the existing cap on `collect_file_definitions`, preventing unbounded memo accumulation for removed files.
- `deleted_files` tracking added to `MirDbStorage` so removed files are explicitly auditable and provide the foundation for future tracked-struct GC.

### Performance

- Variable types stored in `FlowState` and `InferredFileTypes` are now deduplicated via `wrap_var_type`, backed by the existing `intern_or_wrap` pool. Common scalars hit an O(1) fast path; merged types that equal a prior type are also deduplicated, making `Arc::ptr_eq` shortcuts in merge code fire more often.
- `FlowState::new()` no longer allocates a fresh map for the 11 PHP superglobals on every function/method scope entry. Pre-built `Arc` statics are shared via COW, saving ~140 MiB of allocation churn on the project-only analysis pass (measured on Laravel).
- `TemplateParam.bound` changed from `Option<Type>` (176 B inline) to `Option<Arc<Type>>` via `intern_or_wrap`, saving ~36 MiB of allocation churn on the project-only analysis pass.

## [0.34.0] - 2026-06-08

### Added

- `WrongCaseClass` (MIR1009), `WrongCaseFunction` (MIR1010), `WrongCaseMethod` (MIR1011) — new Info-severity diagnostics for case-sensitive identifier references (PHP 8.6 RFC). Covers `new` expressions, static calls, `instanceof`, type hints, `catch` clauses, `extends`/`implements`/`use`-trait declarations, built-in and user-defined functions, instance and static method calls, and `use` import declarations.
- `WrongCaseMethod` now fires when a magic method is defined with wrong casing (e.g. `__CONSTRUCT` instead of `__construct`).
- `InvalidAttribute` (MIR1600) — detects invalid `#[Attribute]` usages: applying `#[Attribute]` to a function, method, property, or parameter; abstract, interface, or trait classes marked as `#[Attribute]`; attribute classes with a private constructor; classes used as attributes without the `#[Attribute]` annotation; attributes applied to elements not matching their declared target; and non-repeatable attributes used more than once on the same element.
- `UndefinedAttributeClass` — emitted when an attribute references a class that does not exist in the codebase.
- `InaccessibleClassConstant` (MIR0011) — emitted when a private or protected class constant is accessed from a context that does not have visibility.
- `DuplicateClass` (MIR1602) — emitted when the same class name is declared more than once within a file, including across braced namespace blocks.
- `ParentNotFound` (MIR0010) — emitted when `parent::` is used (static call, constant access, property fetch, or `parent::class`) inside a class that has no declared parent.
- `OverriddenPropertyAccess` — emitted when a subclass reduces the visibility of an inherited property (`public→protected`, `public→private`, `protected→private`).
- `NullableReturnStatement` — emitted when a function whose return type is non-nullable has a return path that could be null (the non-null part is otherwise compatible with the declared type).
- `InvalidClone` now also fires when cloning a named object whose `__clone()` method is `private` and the caller does not have access.
- `@final` docblock annotation is now treated as equivalent to the native `final` keyword for `InvalidExtendClass` detection.

### Fixed

- `ATTR_TARGET_ALL` corrected from 127 to 63 (the correct sum of the six `TARGET_*` flags). The wrong value accidentally set bit 6 (`IS_REPEATABLE = 64`), making every `#[Attribute]` class without explicit target flags appear repeatable and silently suppressing the "not repeatable" diagnostic.
- `NonStaticSelfCall` no longer suppresses the diagnostic when the class defines `__callStatic`. `__callStatic` only intercepts *undefined* static methods, not explicitly-defined non-static ones.
- `$this` no longer leaks into static arrow functions when resolving captured outer scope.

### Changed

- `FinalClassExtended` renamed to `InvalidExtendClass` to align with Psalm's naming. Update any inline `@mir-suppress FinalClassExtended` annotations to `@mir-suppress InvalidExtendClass`.

## [0.33.0] - 2026-06-05

### Added

- Eager + background vendor indexing with configurable chunk size and memory targets (controlled via `--vendor-memory` flag; defaults to 128 MiB chunks).

### Fixed

- Fixed exponential memory growth when analyzing files with nested conditional branches and repeated dead-write tracking. `FlowState::merge_branches` now deduplicates dead writes instead of concatenating, preventing allocation of gigabytes of memory on large projects like Laravel (NotificationSender.php was OOM-ing at 20GB; now uses 33MB).
- Fixed workspace index singleton cache refresh when analyzing project and lazy-loaded classes, ensuring proper resolution in batch analysis.

### Changed

- Vendor indexing now uses the chunked indexing engine for more predictable memory usage and streaming behavior.

### Performance

- Subtype-check results are now cached per pass (rather than globally) in the body analysis pass, improving cache locality for concurrent analyses.
- Workspace index is now borrowed frozen during body pass analysis, eliminating write-lock contention.
- `PropertyDef` type fields changed from `Option<Type>` to `Option<Arc<Type>>`, reducing per-property overhead by 168 bytes.
- `lazy_load_missing_classes` ingest loop is now parallelized, speeding up vendor class loading in batch mode.

## [0.32.0] - 2026-06-04

### Added

- `TooManyArguments` (MIR0203) is now emitted when arguments are passed to a class that has no explicit `__construct()` method (the implicit constructor accepts zero arguments).
- `InvalidScope` (MIR0001) is now emitted when `$this` is assigned a value outside a class context.
- `InvalidArrayAssignment` (MIR0220) is now emitted when a subscript assignment (`$x[] = …` or `$x[k] = …`) is performed on a scalar type (`int`, `bool`, `float`).
- `InvalidArrayAccess` (MIR0219) is now emitted when subscript access is performed on a scalar type. String subscript indexing (`$str[0]`) remains valid.
- `InvalidPropertyFetch` (MIR0218) is now emitted when a property is accessed on a scalar or non-object type.
- `DirectConstructorCall` (MIR0217) is now emitted for explicit `$obj->__construct()` calls on object instances.
- `NonStaticSelfCall` (MIR0216) is now emitted when `self::`/`static::` is used to call a non-static method in a static context.
- `InvalidStaticInvocation` (MIR0215) is now emitted when a non-static method is called with a concrete class name (`ClassName::method()`) and the class has no `__callStatic`.
- `InterfaceInstantiation` (MIR0709) is now emitted when `new` is used directly on an interface.
- `DeprecatedProperty` (MIR1005) is now emitted when a property marked with `@deprecated` or `#[Deprecated]` is read or written.
- `DeprecatedInterface` (MIR1006) is now emitted when a deprecated interface is implemented.
- `DeprecatedTrait` (MIR1007) is now emitted when a deprecated trait is used.
- `DeprecatedConstant` (MIR1008) is now emitted when a deprecated class constant or enum case is accessed.
- `DeprecatedClass`, `DeprecatedMethod`, and `DeprecatedCall` detection expanded: `#[Deprecated]` is now recognised on user-defined methods and functions; deprecated classes are caught in static calls, constant access, and type hints.
- `DeprecatedMethodCall` is now emitted when cloning an object whose `__clone()` method is deprecated.
- `InvalidCast` is now emitted when `(string)` is applied to a concrete class that does not implement `__toString()`.
- `InvalidCatch` (MIR1503) is now emitted when a `catch` clause names a type that does not extend `Throwable`.
- `ImplicitToStringCast` (MIR1501) is now emitted when a `Stringable` object is passed where a `string` is expected, making the implicit `__toString()` call visible.
- `InvalidOperand` (MIR0213) now covers: arithmetic on non-numeric operands, bitwise operations on objects and arrays, boolean operands in bitwise expressions, boolean increment (`$b++`), and array members in string concatenation.
- `PossiblyNullOperand` (MIR0214) is now emitted when a null value is used as a divisor in `/` or `%`.
- `UnusedForeachValue` is now emitted when the value variable in a `foreach` loop is never read.
- `UnusedVariable` dead-write detection: a variable that is assigned and then overwritten before being read is now flagged.
- `UnusedVariable` is now detected in top-level PHP scripts, not only inside functions and methods.
- `InvalidOverride` (MIR0708) is now emitted when `#[Override]` is applied to a method that has no overridable parent, or whose parent method is `private`.
- `MethodSignatureMismatch` now catches: abstract re-declaration of a concrete method, multi-interface return-type conflicts, by-reference parameter mismatch, overrides that drop parent parameters, and static/non-static mismatch.
- Generic type inference at instantiation: `new Box(5)` now infers `Box<int>` by binding class `@template` parameters from constructor argument types.
- Unannotated generic method returns: methods whose parameters carry template types now resolve concrete return types at call sites without an explicit `@return` annotation.
- `@readonly` docblock annotation on properties is now treated the same as the native `readonly` keyword for the `ReadonlyPropertyAssignment` check.

### Fixed

- `@mixin` property resolution: properties declared on `@mixin` classes are now found via the full inheritance chain, eliminating `UndefinedProperty` false positives for mixin-based patterns.
- Narrowing false positive: possibly-undefined variables no longer cause the `else`/`elseif` branch to be incorrectly marked as unreachable.
- Narrowing in `elseif`/`else` chains: each failed `elseif` condition is now applied as a negative narrowing to the `else` branch.
- `UnusedVariable` false positives in loops: pre-loop writes are cleared after the loop body iterates, preventing them from being re-introduced through the else path.
- `UnusedVariable` false positives for variables passed to `compact()`: those variables are now marked as consumed.
- Return type checking now applies inside anonymous-class methods.
- Stub cache corruption on the second analysis run: `#[serde(skip_serializing_if = "Option::is_none")]` is unsafe with bincode (a non-self-describing format) — the `None` discriminant byte was omitted on write while deserialization still expected it, causing misaligned reads and a runaway allocation. Removed `skip_serializing_if` from the `deprecated` field on `PropertyDef`, `ConstantDef`, `InterfaceDef`, `TraitDef`, and `EnumCaseDef`. Stub cache format version bumped to 4 to invalidate stale on-disk entries.

## [0.31.0] - 2026-06-01

### Added

- Inline issue suppression via source comments: add `// @mir-suppress DiagnosticName` on the offending line (or the line above) to silence a specific diagnostic without affecting others.
- `NonExistentArrayOffset` (MIR0301) is now emitted when a literal string or integer key is accessed on a closed keyed array (`array{foo: int}`) and the key is absent.
- `ParadoxicalCondition` (MIR0404, Warning) is now emitted for duplicate literal values in `switch` cases and `match` arms, where the repeated branch can never be reached.
- Conditionally-declared functions and classes — the `if (!function_exists('foo')) { function foo() {} }` guard pattern used by Laravel helpers, Symfony polyfills, and WordPress pluggable functions — are now indexed. Resolves ~1,608 `UndefinedFunction` false positives on a standard Laravel project.
- All issue locations now carry `line_end`/`col_end` in addition to the existing start position, enabling tighter diagnostic ranges in SARIF, LSP, and playground consumers.

### Fixed

- `UnusedVariable` false positives for variables used as dynamic property or method names (`$this->$var`, `$this->{$var}`, `$this->$method()`).
- `UndefinedClass` false positives for class names used as the argument to `class_exists()`, `interface_exists()`, or `trait_exists()`, and for usages of optional classes inside the guarded true-branch.
- Conditional return types (`@return ($T is null ? X : Y)`) are now resolved at static method call sites, eliminating false `InvalidArgument` errors.
- Short-circuit `&&`/`||` assignments are promoted from possibly-assigned to definitely-assigned when the branch is known to have executed (e.g. the true-branch of `&&`). Reduces `PossiblyUndefinedVariable` false positives in the Laravel benchmark from 31 to 7.
- Composer root detection now skips `vendor/<org>/<pkg>/composer.json` manifests and walks up to the true project root, eliminating ~1,552 `UndefinedClass` false positives on standard Laravel projects.
- `strtr($str, $pairs)` (2-argument array form) no longer fires `TooFewArguments`.
- `TooManyArguments` false positives eliminated when a union type contains a bare `callable` (unknown arity) alongside a typed `TClosure`.
- `UnusedVariable` false positives eliminated for variables read only inside a `finally` block (the save-restore pattern).
- Nested `TConditional` return types (e.g. `($v is null ? array{} : ($v is array ? array<K,V> : array{V}))`) are now recursively resolved rather than returned as opaque conditional types.
- `UndefinedProperty` false positives eliminated for property accesses guarded by `??` or `isset` (e.g. `$this->prop ?? null`).
- `PossiblyUndefinedVariable` false positives eliminated for variables used as the left operand of `??` when the coalesced result is immediately compared against the fallback literal.
- A bare `Closure` type now satisfies a typed `Closure(): T` parameter, eliminating false `InvalidArgument` errors.
- `ingest_file` now evicts dependents' cached analysis when a file's content changes, preventing stale results from being replayed across incremental re-analysis.
- `Enum::Case` and class constant accesses now resolve to the correct type instead of `mixed`.
- `TooManyArguments` false positives eliminated for functions and methods that use `func_get_args()`/`func_num_args()`/`func_get_arg()` in their bodies.
- `InvalidArgument` false positives eliminated for `Stringable` objects passed as `string` parameters in files without `declare(strict_types=1)`.
- `array_keys(array<K, V>)` now returns `list<K>` instead of `list<mixed>`.
- `preg_match` `$matches` parameter is now typed as `array<int, string>` via by-ref write-back.
- `str_replace`/`str_ireplace` return type is narrowed to `string` when the subject is a scalar.
- `hrtime()` return is narrowed to `int|float` when `$as_number` is `true`.
- `NonExistentArrayOffset` is suppressed inside existence-check contexts (`isset`, `??`, `empty`).
- Template parameters in supertype position are now treated as wildcards in `atomic_subtype`, eliminating false `InvalidTemplateParam` diagnostics for union-sub against union bounds.
- `list<T>` is now inferred for the `$arr[] = $v` push notation instead of `array<mixed, T>`.
- `$obj::class` passed as a `class-string<T>` argument no longer fires `InvalidArgument`.
- Nested array assignment (`$arr[$k][] = $v`) now correctly propagates the innermost key type.
- Template parameters inside array types in generic method returns are now correctly resolved.
- Reference index gaps closed: class references recorded at the class identifier in static calls, `self`/`static`/`parent`/`ClassName` constant accesses, and inherited method calls use the declaring class.
- PHP version filtering is now wired into the salsa database so `FileAnalyzer` honours `--php-version` correctly.
- Parser now strips quotes from array shape keys in PHPDoc (`array{'key': T}` parses correctly).
- `mysqli_init()` PHP 8.0 overload (returning `mysqli`) added to stubs.

### Performance

- Peak cold-start memory reduced by ~22 MiB: `MethodDef`/`FunctionDef` inferred return types changed from `Option<Type>` (176 B) to `Option<Arc<Type>>` (8 B); class analysis no longer materializes vendor/stub classes during the analyzed-file decomposition; `mimalloc` installed as the global allocator.

## [0.30.0] - 2026-05-28

### Fixed

- `$argv` and `$argc` are now seeded as predefined globals, eliminating `UndefinedVariable` false positives in CLI scripts.
- Single-star `/* @var $this */` annotations (the form PhpStorm generates for Yii2 view templates) are now recognized in addition to `/**` PHPDoc blocks. Fixes #290.
- `PossiblyUndefinedVariable` false positives eliminated for variables assigned inside `while(true)` and `for(;;)` loops before every `break`. Infinite loops no longer treat the "loop never executes" path as reachable.
- `UnusedVariable` and `UnusedParam` false positives eliminated for variables read only inside a diverging if-branch (one that always `return`s or `throw`s).

### Changed

- Upgraded `php-rs-parser`, `php-ast`, `php-lexer`, and `phpdoc-parser` to 0.15.0. Function and closure bodies are now wrapped in a `Block` type; class/enum/interface/trait members are behind `ClassBody`/`EnumBody` wrappers.

## [0.29.0] - 2026-05-27

### Added

- Cache is now enabled by default without `--cache-dir`. Composer projects cache to `<project-root>/.mir/cache`; other scans use the platform cache directory. Pass `--no-cache` to opt out.
- `@mir-check` inline type assertion directive: annotate a variable with `/** @mir-check $x is SomeType */` in a test fixture to emit `TypeCheckMismatch` if the inferred type does not match, enabling regression tests for type inference.
- Short-circuit `isset`/`!isset` narrowing in `&&` and `||` expressions: `isset($x) && $x->method()` now correctly narrows `$x` to non-null inside the right-hand side.
- `InvalidStringClass` diagnostic: emitted instead of `UndefinedClass` when a dynamic class expression (`new $var`, `$var::method()`) is not a valid `class-string`. String literal arguments to `class-string` parameters are now validated.
- `TCallableString` atomic type for proper callable-string validation.
- Variance checking for generic return types: a method return type that widens its parent's generic parameter now emits a diagnostic.

### Fixed

- **Template bounds (FQN resolution)**: eliminated ~2,100 false-positive `InvalidTemplateParam` and `InvalidArgument` diagnostics caused by bare class names in `@template T of …` bounds not being namespace-qualified. Fixes cover all definition collectors (class, interface, trait, function, method), intersection bounds, `@var` and property type annotations, and generic type arguments.
- **Template conditional returns**: `@return (T is null ? X : Y)` now parses and resolves correctly at call sites. When T is already bound in the substitution, the conditional collapses to the correct branch. When the discriminator is nullable-but-not-only-null, the conditional widens to `X|Y` instead of emitting a false positive.
- **Intersection types**: intersection-typed values are now recognized as subtypes of their parts and of `object`, eliminating companion `InvalidArgument` false positives for functions like `get_class()`. `InvalidArgument` is also suppressed when a parameter type contains templates within an intersection.
- **Template inference**: `T` is now correctly inferred from `class-string<T>` arguments, `Closure`, `callable`, and intersection-typed parameters. Template bounds now check inheritance chains. Array-key pseudo-type and `TKeyedArray` are recognized in template binding.
- **Array types**: empty keyed arrays (`array{}`) are folded into matching generic arrays in unions, eliminating `|array{}` noise from loop-built arrays. Array key types are now preserved in `$arr[$key] = $val` assignments, fixing ~62 false-positive `InvalidReturnType` diagnostics. Mutual-reference array loops no longer cause an infinite hang during inference.
- **PHP built-ins**: `array_walk`, `array_walk_recursive` 3rd parameter is now optional; `mt_rand`/`rand` parameters are now optional. Fixes ~30 `TooFewArguments` false positives. `array_map` with multiple arrays now accepts a callback with matching arity instead of requiring arity 1, fixing ~62 false positives.
- **Enum built-ins**: `from()`/`tryFrom()` are now synthesized with one parameter, eliminating `TooManyArguments` false positives.
- **Narrowing**: `UndefinedVariable` is no longer emitted for variables on the left-hand side of `??` and `??=`. `assigned_vars` is now correctly restored after `isset`-narrowed branches.
- **Column numbers**: diagnostic column numbers are now 1-indexed (previously 0-indexed). Any tooling that parses mir output should update accordingly.
- **Stubs**: user-defined files now consistently override native stub definitions in the symbol index, eliminating non-deterministic false positives when shadowing PHP built-in names.
- `self::CONST` references in method parameter defaults now correctly emit `UndefinedConstant` when the constant does not exist.
- First-class callable syntax (`SomeClass::method(...)`) now resolves to a typed `TClosure` instead of an untyped callable.
- `InvalidStringClass` false positives eliminated for object expressions on the left of `::` (e.g. `$obj::CONST`).

### Changed

- `ProjectAnalyzer` is replaced by `AnalysisSession` in the public API. The new type consolidates project setup and analysis into a single entry point.
- Stub loading is now fully lazy: stubs for a PHP version are loaded on first reference rather than at startup, reducing cold-start memory for projects that use only a subset of built-ins.

## [0.28.0] - 2026-05-17

### Added

- Composer plugin type: `composer require jorgsowa/mir` now triggers the binary download automatically without requiring manual script wiring. The `composer.json` type field is set to `composer-plugin`, and a `Plugin` class registers the install/update event handler.

### Fixed

- Composer installer now embeds the target triple in the version marker, preventing a binary installed on one platform (e.g. macOS) from being reused on a different one (e.g. Linux in Docker). The shim error message for `proc_open` failures now mentions a possible architecture mismatch.
- Broken relative links in the error codes reference table (`./` → `../`) that caused 404s when navigating from the codes page to individual issue pages.
- Documentation corrections for `ImplicitToStringCast`, `InvalidCast`, `UndefinedClass`, `InvalidScope`, `DeprecatedMethod`, and `DeprecatedMethodCall` issue pages. Added missing `UndefinedTrait` (MIR0009) documentation page.

## [0.27.0] - 2026-05-17

### Added

- Stable `MIR####` error codes for every issue variant, organized into 16 category bands. Codes surface in `Display` output in rustc style: `error[MIR0005] UndefinedClass: ...`. The `name()` method is unchanged and remains the suppression and SARIF rule key.
- `UndefinedTrait` (MIR0009) diagnostic: emitted when a `use` statement references a name that does not exist in the codebase.
- `InvalidTraitUse` now also emitted when the used name resolves to a class, interface, or enum instead of a trait. Per-`use`-statement source locations are stored in `ClassStorage` and `ClassNode` so diagnostics point at the trait name in the `use` statement.
- php-rs-parser 0.13.0: parse errors now carry precise source locations via `err.span()` instead of hardcoded line 1 col 0; `ForbiddenWarning` diagnostics emit at `Severity::Warning` and do not block semantic analysis.

### Fixed

- Literal integer (`1`, `42`, `-3`) and quoted-string (`'foo'`, `"bar"`) types in docblock annotations now parse as `TLiteralInt` / `TLiteralString` instead of `TNamedObject`, making `@return 2|3` and similar annotations work correctly.
- `@return` / `@param` docblocks written on the line preceding a standalone function declaration (rather than attached as an AST `doc_comment`) are now applied, matching the existing behavior for class methods.
- `@method` docblocks on traits, interfaces, and enums are now honored. Previously `add_docblock_members` was only called for classes, silently dropping virtual method declarations on other symbol kinds. `@method`-added methods carry `is_virtual: true` and are excluded from `UnimplementedInterfaceMethod` checks.
- `UnusedVariable` now reports the correct source location for variables first assigned via array push (`$arr[] = value`), `static $var`, or `global $var` (previously fell back to line 1, col 0).
- `global $var` assignments are now treated as externally observable side effects (matching by-reference parameter semantics), eliminating false-positive `UnusedVariable` diagnostics on global variable writes.
- `Union::intersect_with` now returns `never()` when no types overlap between the subject and the arm condition, preventing false-positive method/property errors in match arm bodies. `Union::add_type` now absorbs `never` into non-empty unions (`T | never = T`).
- Pending reference locations are now drained into `RefLocAccumulator` inside `analyze_file` (Salsa), fixing reference tracking in the incremental analysis path.

### Changed

- `MissingThrowsDocblock` is now suppressed by default for `RuntimeException` and `LogicException` descendants (PHP's "unchecked" exceptions). Both direct `throw` statements and transitive `@throws` propagation are filtered. The suppression list is configurable via the new `suppressed_issue_kinds` API.
- `find_dead_code: bool` on `ProjectAnalyzer` replaced with `suppressed_issue_kinds: HashSet<String>` and a centralized `apply_issue_suppressions()` post-filter applied on every analysis path including the cache-hit path.
- Removed the `instanceof` operator-precedence workaround from `narrowing.rs`; php-rs-parser 0.13.0 correctly parses `!$x instanceof C` as `!($x instanceof C)`.

### Dependencies

- Bumped php-rs-parser, php-ast, php-lexer, phpdoc-parser `0.12.1` → `0.13.0`.

## [0.26.0] - 2026-05-15

### Performance

- Persistent Pass-1 cache (`StubSliceCache`): when a cache directory is configured (`ProjectAnalyzer::with_cache`, `AnalysisSession::with_cache_dir`, or `--cache-dir`), each file's `StubSlice` is stashed in `<cache_dir>/stubs/<hh>/<full_hash>.bin` using a content-hash key, a bincode binary encoding, and atomic tempfile-and-rename writes. On a warm cache, files skip parse and definition collection (≈95% of the per-file cost on Laravel) and the cached slice is ingested directly. Cache header is version-gated by `CARGO_PKG_VERSION`, the on-disk format version, and the target PHP version, so cached data is automatically invalidated across mir or PHP-version upgrades.
- Both the batch path (`ProjectAnalyzer::collect_types_only`, exercised by the CLI for vendor warmup) and the per-file LSP path (`AnalysisSession::ingest_file` via `SharedDb::collect_and_ingest_file`) consult the cache. Measured on `laravel/framework v11.44.7` (10,188 vendor files, M-series Mac), independently verified hit counters (`10,185 hits / 0 misses` on warm, the 3-file delta is files mir skips for parse errors and is excluded from caching):
  - Vendor batch collection: cold 2,224 ms / 2,822 MiB churn → warm 1,440 ms / 525 MiB churn (−35% wall, −81% churn). Repeated runs land in a −30% to −46% wall-time band depending on the OS page-cache state of the underlying vendor tree.
  - LSP-style serial `ingest_file` storm via `AnalysisSession`: cold 5,476 ms → warm 3,720 ms (−32% wall). The serial path is bottlenecked by Salsa write-lock + ingest cost the cache doesn't address.
- Cache misses (or files with parse / collector errors) skip the write-back so future runs re-parse them; cache hits restore the file path field from the lookup argument so the on-disk encoding never carries a machine-specific absolute path.
- `ProjectAnalyzer::{with_cache_dir,set_cache_dir}` and `AnalysisSession::{with_cache,with_cache_dir}` now `debug_assert` they are called before any file is ingested — late attachment would silently reset the shared database and discard prior Pass-1 work.

### Dependencies

- Bumped all transitive crates within their compatible semver ranges (`cargo update`), including the `php-rs-parser` / `php-ast` / `php-lexer` / `phpdoc-parser` stack from `0.12.0` → `0.12.1`.
- Bumped `quick-xml` `0.39` → `0.40` in `mir-analyzer`.
- Replaced `postcard` with `bincode 1.3.3` for the `StubSliceCache` on-disk format. `postcard` pulled `heapless` → `atomic-polyfill` (RUSTSEC-2023-0089); `bincode v2` was tried next but is itself flagged unmaintained (RUSTSEC-2025-0141). `bincode 1.3.3` carries no advisory and is explicitly called "complete" by its authors. Cache on-disk format version bumped to 2 so existing v2-encoded entries are treated as misses.

## [0.25.0] - 2026-05-15

### Performance

- Pass 2 reference-location recording now uses per-worker staging buffers (`PendingRefLocs`) instead of writing directly to shared `Arc<Mutex<...>>` maps. Workers accumulate locations in an isolated `parking_lot::Mutex<Vec<RefLoc>>` and a single serial commit drains them with one lock acquisition per map. Pass 2 wall-clock variance reduced from 28–240 ms (8×) to 43–56 ms (±25%) on 12 threads.

### Fixed

- `analyze_dependents_of()` now returns the correct dependent set after a symbol is deleted or renamed. Previously, files referencing a now-gone symbol were silently dropped because `dependency_graph()` routed edges through `symbol_defining_file()`, which returns `None` for deleted symbols. Three coordinated fixes: a `file_to_defined_symbols` forward index for O(1) definition lookup on removal; a `symbol_referencers` reverse index that survives symbol deletion; and a `stale_defined_symbols` accumulator in `AnalysisSession` that feeds deleted symbols' referencers back into the BFS.

## [0.24.0] - 2026-05-15

### Performance

- O(1) parameter deduplication: replaced linear Vec scan with FxHashMap for ~20% faster stub ingestion on large vendor sets. Deduplication now runs in parallel within rayon Pass 1 instead of serializing the collector.
- RwLock-based atomic counter writes for Salsa db updates, reducing lock contention during batch analysis and improving 12-thread scaling.
- `file_references` forward index added to `MirDb`: `dependency_graph()` cost reduced from O(S×R) to O(E) (files × edges), eliminating full-table scans during incremental re-analysis.
- In-memory always-on reverse dependency map (`structural_dependents_of`) for O(D) BFS over structural dependencies (imports, class hierarchy, type hints) without requiring disk cache.

### Fixed

- Reference location recording now complete at all five previously-missing call sites: `instanceof`, `catch`, `::class`, `::CONST`, and type-hint declarations. Files referencing a class only via these constructs are now correctly visible to the incremental dependency graph and `analyze_dependents_of()`.

## [0.23.0] - 2026-05-14

### Added

- Type narrowing for `get_class($obj) === 'ClassName'` comparisons, enabling precise type refinement when class identity is verified.
- `is_resource()` type guard for completeness in the type narrowing system.
- Parallel Salsa pre-sweep inference pass in batch path, replacing sequential Pass 2 driver with direct rayon-based inference for improved throughput.
- Type narrowing for `$var === SomeClass::class` comparisons, refining object types when matched against class constants.

### Fixed

- Bare-FQN references (e.g., `new \Service()`, `\Helper::go()`) now correctly wired into the incremental dependency graph so `analyze_dependents_of()` returns files referencing classes via unqualified absolute paths.

### Changed

- Refactored database module structure: `source_files` map moved from SharedDb tuple into MirDb for clearer ownership.
- Lazy-load optimization: avoid redundant full scans of class inheritance chains when loading missing classes.

## [0.22.0] - 2026-05-12

### Added

- `AnalysisSession::class_issues_for()`: exposes cross-file class diagnostics (abstract-method gaps, override violations, circular inheritance) so LSP consumers can retrieve the complete diagnostic picture alongside `analyze_dependents_of()` without accessing `ClassAnalyzer` directly.

## [0.21.2] - 2026-05-12

### Fixed

- `@template T as Bound` syntax now parsed correctly (previously only `@template T of Bound` was recognized), enabling proper type narrowing for templates declared with the `as` keyword.
- Callable/closure return types in `@return` annotations (e.g., `@return \Closure(): T`) now correctly capture the return type after the colon, fixing false `MixedMethodCall` diagnostics when template parameters were used as closure return types.

## [0.21.1] - 2026-05-09

### Fixed

- `cargo-deny` configuration format migration to version 2.

## [0.21.0] - 2026-05-09

### Added

- Tier 1 & 2 parser optimizations: pre-sized arena allocators and parallel user stub discovery for improved cold-start performance (25-40% improvement expected).

### Fixed

- `cargo-deny` configuration format corrected to use proper advisories section syntax.
- Security audit findings: eliminated unwrap calls and unsafe UTF-8 conversions.
- Panic on empty generic type parameters in docblock parsing.
- Outdated lock poison `.expect()` calls replaced with proper error handling.
- Template parameter bounds preservation and improved generic type narrowing.
- MixedClone detection for unconstrained template parameters.
- Missing stubs directory safety check in build.rs.
- Soft stub fallback version-gating for both functions and classes.

### Changed

- Refactored AST-based stub discovery in FileAnalyzer for clarity and performance.
- Split db.rs into focused sub-modules for maintainability.
- Improved code quality with centralized test utilities.
- Eliminated HashMap/HashSet clones in cache flush hot paths.
- Reduced string clone allocations in hot paths.
- Replaced std::sync::Mutex with parking_lot::Mutex to eliminate poison panics.

### Performance

- Parallelized fixture discovery in build script.

## [0.20.0] - 2026-05-08

### Added

- Session-based per-file analysis API (`AnalysisSession` + `FileAnalyzer`) for incremental, file-scoped analysis suitable for LSP-style consumers.
- `mir_analyzer::location_from_span(span, file, source, source_map) -> Location`: public free function that converts a parser `Span` (byte-offset range) to the crate's `Location` type (1-based lines, 0-based codepoint columns), so consumers can translate Pass-2 spans to their own protocol's position format without re-implementing column math.
- Soft fallback for unknown stubs: when Pass 2 would emit `UndefinedFunction` / `UndefinedClass` for a name the build-time stub index recognises as a real PHP built-in, the diagnostic is suppressed. Defends against lazy-stub timing races (auto-discovery scanner false negatives, essentials-only sessions without auto-discovery, mid-ingest reads). Genuinely unknown names still emit.
- Concurrent-read benchmark: N reader threads call `definition_of()` in a tight loop while a writer continuously re-ingests a fixture, reporting wall time per fixed-size batch for 1 / 4 / 8 readers. Surfaces real contention characteristics under flat-out write pressure (per-read latency: 324ns @ 1 reader, 1.4µs @ 4, 1.9µs @ 8); realistic LSP edit cadence stays at the 324ns figure.
- `MixedClone` issue type: detects `clone` / `clone with` expressions on `mixed`-typed values in `ExpressionAnalyzer`.

### Fixed

- `@var` annotation narrowing now applies to global-scope statements, not just function bodies. Previously `analyze_stmt()` (used for top-level statements) skipped the pre/post narrowing that `analyze_stmts()` performed for function bodies, so `@var` had no effect at global scope. Fixes `global_with_var_no_indent`, `function_with_var`, and `invalid_mixed_clone` fixtures.

### Changed

- Analyzer boilerplate simplifications:
  - `Union::core_type()` collapses 10+ chained `remove_null().remove_false()` call sites in type-checking logic.
  - `DefinitionCollector::parse_docblock_from_node_or_preceding()` consolidates the "check `doc_comment`, fall back to preceding docblock" pattern repeated 11+ times across class/trait/interface collectors.
  - `StatementsAnalyzer::span_to_location()` replaces 7 instances of verbose span-to-location computation in flow analysis.

## [0.19.0] - 2026-05-07

### Added

- Trait method undefined function detection: diagnostics now detect when trait methods reference undefined functions, improving visibility into broken trait implementations.
- Enhanced inheritance chain checking for magic methods (`__get`, `__invoke`): full ancestor chain is now properly examined, catching edge cases where magic methods are defined in distant parent classes.

### Fixed

- Magic method resolution (`__get`, `__invoke`) now checks the complete ancestor chain instead of stopping at the immediate parent, fixing false negatives where inherited magic methods were not detected.
- Unused method tests now properly handle collateral errors, improving test reliability and reducing false positives in fixture validation.

## [0.18.0] - 2026-05-06

### Added

- `AbstractInstantiation` diagnostic to detect attempts to instantiate abstract classes via `new ClassName()`.

### Fixed

- Closure `use()` clause validation: now detects undefined variables referenced in closure use() clauses. Example: `use ($i)` will report `UndefinedVariable` if `$i` is not defined in the parent scope.
- Mixin method resolution with generics: docblock `@mixin Foo<T>` annotations now correctly resolve to class `Foo` instead of attempting to look up a non-existent class named `Foo<T>`.
- All 17 `undefined_variable` fixture tests now pass with correct line/column/message expectations.
- All 15 `undefined_constant` fixture tests now pass with correct line/column/message expectations.

## [0.17.3] - 2026-05-05

### Performance

- Deduplicate parameter types across all function/method signatures via `Arc<Union>` interning, eliminating redundant type allocations.
- Resolve function node once per call site instead of twice, reducing redundant database lookups.
- Use `SimpleType` for atomic function parameters, reducing type envelope overhead.
- Deduplicate return types via `Arc<Union>` interning for all callables.
- Deduplicate parameter lists across vendor method signatures, further reducing memory footprint.
- Skip re-caching `StubSlice` in Salsa during vendor collection, improving vendor ingestion performance.

## [0.17.2] - 2026-05-04

### Fixed

- The published `mir-analyzer` crate is no longer shipped with an empty stub set. The `stubs/` directory lived at the workspace root, outside the package, so `cargo package` excluded it; downstream consumers (e.g. `php-lsp`) saw `STUB_FILES = &[]` and every PHP built-in resolved as `UndefinedFunction` / `UndefinedClass`. Stubs now live inside the crate at `crates/mir-analyzer/stubs/` and are included in the published artifact. `build.rs` panics if the directory is missing, and a new `tests/packaging.rs` test asserts `cargo package --list` includes `stubs/Core/Core.php` plus the rest of the stub set — closing the publish-time gap.
- Built-in function and class lookups are now case-insensitive, matching PHP semantics. `Restore_Error_Handler()`, `RESTORE_ERROR_HANDLER()`, `new arrayobject([])`, and `new ARRAYOBJECT([])` no longer produce false-positive `UndefinedFunction` / `UndefinedClass` diagnostics. Implemented as side indices on `MirDb` (`function_node_keys_lower`, `class_node_keys_lower`) so the canonical-FQN storage that `active_*_node_fqns`, `function_count`, `type_count`, and `clear_file_references` depend on is unchanged. Constants remain case-sensitive (PHP semantics).

## [0.17.1] - 2026-05-03

### Fixed

- Unqualified class names in namespaced files no longer silently fall back to the global namespace when the namespaced class is missing. PHP only does that fallback for functions and constants; mir's `resolve_name_via_db` was incorrectly extending it to classes, masking real `UndefinedClass` bugs.
- Composer autoload parsing now covers `psr-0`, `classmap`, and `files` in addition to `psr-4`, for both project `composer.json` and each package in `vendor/composer/installed.json`. Vendor packages that expose global helpers via `autoload.files` (Symfony polyfills, Laravel helpers, ramsey/uuid bootstrap, etc.) and classmap-only packages no longer produce false-positive `UndefinedFunction` / `UndefinedClass` diagnostics.

## [0.17.0] - 2026-05-03

### Removed

- `mir_codebase::Codebase` struct, `CodebaseBuilder`, `codebase_from_parts`, and the internal `Interner` module. The salsa db (`MirDb`) is the single source of truth for class/method/property/constant metadata, per-file imports/namespaces, global vars, and reference tracking. The `mir-codebase` crate now exports only the serializable storage types (`StubSlice`, `*Storage`, `FnParam`, `TemplateParam`, `Visibility`, `Location`). **Breaking** for library consumers that imported `mir_codebase::Codebase`.
- `ProjectAnalyzer::codebase()` accessor (already removed in 0.16.x perf work; the Codebase deletion completes the cleanup).
- `mir-codebase` no longer pulls in `dashmap` or `thiserror`.

### Performance

- Hot-path Salsa db lookup tables (`class_nodes`, `function_nodes`, `method_nodes`, `property_nodes`, `class_constant_nodes`, `global_constant_nodes`, `file_namespaces`, `file_imports`, `global_vars`, `symbol_to_file`, `reference_locations`) and the ancestor-walk visited sets in `class_ancestors` / `lookup_method_in_chain` / `method_is_concretely_implemented` now use `FxHashMap` / `FxHashSet` instead of std `HashMap` / `HashSet`. Eliminates the per-ancestor `String` allocation in `class_ancestors` (now reuses the existing `Arc<str>`). ~7% reduction in user CPU time on the Laravel `src/` benchmark.

## [0.16.1] - 2026-05-01

### Fixed

- CLI Composer detection now walks up from a single explicit file path to find the nearest `composer.json`, so root config files such as `.php-cs-fixer.php` can resolve project PSR-4 namespaces instead of reporting false-positive `UndefinedClass` diagnostics.

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
