# mir — PHP Static Analyzer in Rust
## Implementation Roadmap

> A fast PHP static analyzer built in Rust.
> Target: fast, incremental, parallel analysis with a sound type system.

---

## Implementation Status

| Milestone | Status |
|-----------|--------|
| M0 — Workspace Bootstrap | ✅ Complete |
| M1 — Type System | ✅ Complete |
| M2 — Parser Wrapper | ✅ Complete (using `php-ast`/`php-rs-parser` directly) |
| M3 — Stubs (`mir-stubs`) | ✅ Complete — `mir-stubs` crate; ~180 builtins + Exception hierarchy + core interfaces as `FunctionStorage`/`ClassStorage`/`InterfaceStorage`; no `.phpstub` file parsing (Rust-native stubs) |
| M4 — Codebase Registry | ✅ Complete |
| M5 — Pass 1: Definition Collection | ✅ Complete |
| M6 — Issue System | ✅ Complete |
| M7 — Expression Analyzer | ✅ Complete |
| M8 — Statement Analyzer | ✅ Complete |
| M9 — Call Analyzer | ✅ Complete |
| M10 — Type Narrowing | ✅ Complete |
| M11 — Class Analyzer | ✅ Complete |
| M12 — Loop Analysis | ✅ Complete |
| M13 — Generic Types | ✅ Complete |
| M14 — Pass 2: Body Analysis | ✅ Complete |
| M15 — Configuration | ❌ Not started |
| M16 — CLI | ⚠️ Partial — progress bar (`indicatif`), `--no-progress`, `--verbose`, `--php-version`, JUnit XML, SARIF added; `--set-baseline`/`--update-baseline` still missing (needs M15) |
| M17 — Cache Layer | ✅ Complete |
| M18 — Dead Code Detection | ✅ Complete |
| M19 — Taint Analysis | ✅ Complete |
| M20 — Plugin System | ❌ Not started |

### Post-roadmap improvements (implemented beyond original plan)

| Feature | Description |
|---------|-------------|
| `use` import resolution | Per-file import table stored in `Codebase`; class/function names resolved via `resolve_class_name` during Pass 2 |
| Namespace function fallback | `strlen()` in `App\Ns` tries `App\Ns\strlen` then falls back to global — matches PHP's resolution order |
| `match` expression | Arms forked, subject var narrowed to arm literal type, contexts merged |
| Closure/arrow function body analysis | Full `StatementsAnalyzer` run inside closure; `use` vars captured with taint; return type inferred |
| Named arguments (PHP 8) | `check_args` maps named args to params by name; positional fill the rest |
| `instanceof` narrowing with use resolution | Class name resolved to FQCN before narrowing via updated `narrow_from_condition` signature |
| Switch subject narrowing | Subject variable narrowed to literal type per case arm (`int`, `string`, `bool`, `null`) |
| Array destructuring types | `[$a, $b] = $arr` assigns value type from `TArray`/`TList` to each target variable |
| Readonly property enforcement | Assignment to `readonly` property outside constructor emits `ReadonlyPropertyAssignment` |
| `@psalm-suppress` per-statement | Preceding docblock scanned; issues emitted in the statement are suppressed by name |
| Protected visibility via inheritance | `check_method_visibility` uses `extends_or_implements` to allow calls from subclasses |
| Constructor arg checking | `check_constructor_args` called from `new Foo(...)` expression handler |
| `inside_constructor` tracking | `Context::for_method` accepts `inside_constructor: bool`; set true for `__construct` |
| `@throws` validation | Thrown type checked against `Throwable` hierarchy; emits `InvalidThrow` if not valid |
| PHP stdlib builtins | Moved from ~350-line match in `call.rs` to `mir-stubs` crate: ~180 `FunctionStorage` entries with correct param counts + return types; Exception hierarchy + core interfaces as `ClassStorage`/`InterfaceStorage` |
| Progress bar + output formats | `indicatif` progress bar driven by `on_file_done` callback; JUnit XML and SARIF 2.1.0 output formats added to CLI |

---

## Architecture Overview

```
Source Files
    │
    ▼
[1] File Discovery & Config (mir.xml)
    │
    ▼
[2] Parsing  (php-parser-rs → normalized AST)
    │
    ▼
[3] Pass 1 — Definition Collection  (parallel per file)
    │  Classes, interfaces, traits, enums, functions, constants
    ▼
[4] Codebase Finalization
    │  Resolve inheritance chains, build method dispatch tables
    ▼
[5] Pass 2 — Body Analysis  (parallel per function/method)
    │  Type inference, narrowing, call checking, branch merging
    ▼
[6] Issue Collection & Reporting
    │  Deduplicate, filter by config level, format output
    ▼
[7] Cache Write
    │  Persist per-file AST + type info keyed by source hash
    └─ (fast-path: cache hit skips passes 2-6 for unchanged files)
```

---

## Workspace Structure

```
mir/
├── Cargo.toml                  # workspace
├── ROADMAP.md
├── README.md
├── mir.xml                     # default config (future — M15)
├── crates/
│   ├── mir-types/              # Type system: Union, Atomic, templates
│   ├── mir-issues/             # Issue kinds, severity, reporting
│   ├── mir-codebase/           # Global symbol registry
│   ├── mir-parser/             # PHP → AST, normalization, span tracking
│   ├── mir-stubs/              # Built-in PHP stubs (functions, classes, interfaces)
│   ├── mir-analyzer/           # Analysis engine (statements, expressions, calls)
│   ├── mir-cache/              # Incremental cache layer
│   └── mir-cli/                # Binary entrypoint (clap)
```

---

## Milestones

### M0 — Workspace Bootstrap ✅
**Goal:** Compilable workspace with all crates stubbed out.

- [x] Init Cargo workspace
- [x] Create all crate skeletons (`lib.rs` + `Cargo.toml` per crate)
- [x] Wire up `mir-cli` binary that prints version
- [ ] Set up CI: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`
- [x] Add `php-parser-rs` as a dependency; confirm it parses a simple PHP file

**Exit criteria:** `cargo build --workspace` succeeds; `mir --version` runs.

---

### M1 — Type System (`mir-types`)
**Goal:** A complete, correct type algebra. Everything downstream depends on this.

#### 1.1 Atomic Types

```rust
pub enum Atomic {
    // Scalars
    TString,
    TLiteralString(Arc<str>),
    TClassString(Option<String>),   // class-string<T>
    TNumericString,
    TInt,
    TLiteralInt(i64),
    TIntRange { min: Option<i64>, max: Option<i64> },
    TFloat,
    TLiteralFloat(f64),
    TBool,
    TTrue,
    TFalse,
    TNull,
    TVoid,
    TNever,
    TMixed,
    TScalar,
    TNumeric,

    // Objects
    TObject,
    TNamedObject { fqcn: Arc<str>, type_params: Vec<Union> },
    TStaticObject { fqcn: Arc<str> },   // `static`
    TSelf { fqcn: Arc<str> },           // `self`

    // Callables
    TCallable { params: Option<Vec<FnParam>>, return_type: Option<Box<Union>> },
    TClosure  { params: Vec<FnParam>, return_type: Box<Union>, this_type: Option<Box<Union>> },

    // Arrays
    TArray    { key: Box<Union>, value: Box<Union> },
    TList     { value: Box<Union> },
    TNonEmptyArray { key: Box<Union>, value: Box<Union> },
    TNonEmptyList  { value: Box<Union> },
    TKeyedArray    { properties: IndexMap<ArrayKey, KeyedProperty>, is_list: bool },

    // Generics / meta-types
    TTemplateParam { name: Arc<str>, as_type: Box<Union>, defining_entity: DefiningEntity },
    TConditional   { subject: Box<Union>, if_true: Box<Union>, if_false: Box<Union> },
    TInterfaceString,
    TEnumString,
}
```

#### 1.2 Union Type

```rust
pub struct Union {
    pub types: SmallVec<[Atomic; 2]>,
    pub possibly_undefined: bool,   // variable may not be set
    pub from_docblock: bool,        // originated from annotation, not inference
    pub ignore_nullable_issues: bool,
    pub ignore_falsable_issues: bool,
}
```

#### 1.3 Core Operations

- [x] `Union::single(atomic)` / `Union::empty()` / `Union::mixed()`
- [x] `Union::nullable(atomic)` — shorthand for `T|null`
- [x] `Union::is_nullable() -> bool`
- [x] `Union::remove_type<F>()` / `Union::filter_types<F>()`
- [x] `Union::merge(a, b) -> Union` — used at branch join points
- [x] `Union::intersect_with(other) -> Union` — type narrowing
- [x] `Union::is_subtype_of_simple(other) -> bool`
- [x] `Union::can_be_falsy() -> bool`
- [x] `Union::can_be_truthy() -> bool`
- [x] `Union::narrow_to_truthy() -> Union`
- [x] `Union::narrow_to_falsy() -> Union`
- [x] `Union::narrow_instanceof(class) -> Union`
- [x] `Union::narrow_to_is_type(is_fn: &str) -> Union`
- [x] `Union::substitute_templates(map) -> Union`
- [x] `Display` impl for human-readable type strings

#### 1.4 Subtype Rules

- [x] Implement `atomic_subtype` for all major atomic pairs
- [ ] Handle covariance/contravariance for generic params
- [x] Handle `static` / `self` resolution in subtype checks
- [ ] Property tests: transitivity, reflexivity, `never <: T <: mixed`

**Exit criteria:** 200+ unit tests on type operations pass.

---

### M2 — Parser Wrapper (`mir-parser`) ✅
**Goal:** Reliable PHP → normalized AST with source spans on every node.

- [x] Wrap `php-rs-parser`; expose parse utilities via `mir-parser`
- [x] Normalize name resolution: `name_to_string`, `type_from_hint`
- [x] Source spans via `span_to_line_col`
- [x] Handle PHP 8.0 / 8.1 / 8.2 / 8.3 syntax (match, enums, readonly, named args)
- [x] Docblock parser: `find_preceding_docblock` + tag extraction for `@param`, `@return`,
      `@var`, `@throws`, `@template`, `@psalm-suppress`
- [x] Error recovery: parse errors collected as `ParseError` issues without aborting

**Docblock annotation types to parse:**

| Tag | Purpose |
|-----|---------|
| `@param Type $name` | Parameter type override |
| `@return Type` | Return type override |
| `@var Type` | Variable type annotation |
| `@throws ClassName` | Declares thrown exception |
| `@template T` | Declares a type parameter |
| `@template T of U` | Bounded type parameter |
| `@extends Class<T>` | Generic parent |
| `@implements Iface<T>` | Generic interface |
| `@psalm-assert Type $var` | Type assertion |
| `@psalm-assert-if-true Type $var` | Conditional assertion |
| `@psalm-pure` | Marks function as pure |
| `@psalm-immutable` | Marks class as immutable |
| `@psalm-ignore-nullable-return` | Suppress nullable |
| `@psalm-suppress IssueName` | Suppress specific issue |
| `@deprecated` | Mark as deprecated |
| `@internal` | Mark as internal |
| `@readonly` | Mark property as readonly |

**Exit criteria:** Parses all files in a mid-size open-source PHP project without panicking.

---

### M3 — Stubs (`mir-stubs`) ✅ Complete
**Goal:** Type-correct definitions for all PHP built-in functions, classes, and constants.

#### Implementation

Rust-native stubs compiled directly into the binary via `crates/mir-stubs`. `load_stubs(codebase)`
is called at the start of `ProjectAnalyzer::analyze()` before Pass 1, so user code can override
any stub by defining a symbol with the same name.

**What's registered:**

- **~180 built-in functions** across: string, array, math, type-checking, JSON, file I/O,
  date/time, output buffering, session, headers, error handling, misc — each as a full
  `FunctionStorage` with correct required/optional/variadic/byref param counts and return types.
- **Exception hierarchy:** `Exception`, `Error`, `RuntimeException`, `LogicException`,
  `InvalidArgumentException`, `BadMethodCallException`, `BadFunctionCallException`,
  `OverflowException`, `UnderflowException`, `OutOfRangeException`, `OutOfBoundsException`,
  `RangeException`, `LengthException`, `DomainException`, `UnexpectedValueException`,
  `TypeError`, `ValueError`, `ArithmeticError`, `DivisionByZeroError`, `ParseError`
- **Core interfaces:** `Throwable`, `Stringable`, `Countable`, `Traversable`, `Iterator`,
  `IteratorAggregate`, `ArrayAccess`, `DateTimeInterface`, `JsonSerializable`
- **Other classes:** `stdClass`, `Closure` (with `bind`/`bindTo`/`call`), `Generator`,
  `DateTime`, `DateTimeImmutable`, `SplStack`, `SplQueue`, `SplDoublyLinkedList`

The hardcoded ~350-line match block in `call.rs` has been removed. Builtins now flow through
the normal codebase lookup, enabling argument-count checking on all built-in functions.

- [x] Stubs loaded into `Codebase` before user-code analysis
- [x] Remove inline builtin match arms from `call.rs`
- [x] Exception hierarchy and core interfaces stubbed
- [ ] `.phpstub` file parsing — future enhancement
- [ ] PDO, PDOStatement, mysqli OO API stubs
- [ ] Validate: every stub function resolves without type errors

**Exit criteria:** `is_string($x)` resolves to `bool`; `array_map(fn, array)` resolves to `array`. ✅

---

### M4 — Codebase Registry (`mir-codebase`) ✅
**Goal:** Thread-safe, queryable registry of all symbols across the project.

#### Data Structures

```rust
pub struct Codebase {
    pub classes:    DashMap<FqcnKey, Arc<ClassStorage>>,
    pub interfaces: DashMap<FqcnKey, Arc<InterfaceStorage>>,
    pub traits:     DashMap<FqcnKey, Arc<TraitStorage>>,
    pub enums:      DashMap<FqcnKey, Arc<EnumStorage>>,
    pub functions:  DashMap<FqcnKey, Arc<FunctionStorage>>,
    pub constants:  DashMap<FqcnKey, Union>,

    // Computed during finalization
    pub class_parents:    DashMap<FqcnKey, Vec<FqcnKey>>,   // all ancestors
    pub class_interfaces: DashMap<FqcnKey, Vec<FqcnKey>>,   // all interfaces
    pub method_tables:    DashMap<FqcnKey, IndexMap<MethodKey, Arc<MethodStorage>>>,
}

pub struct ClassStorage {
    pub fqcn: Arc<str>,
    pub parent: Option<Arc<str>>,
    pub interfaces: Vec<Arc<str>>,
    pub traits: Vec<Arc<str>>,
    pub own_methods: IndexMap<MethodKey, Arc<MethodStorage>>,
    pub own_properties: IndexMap<PropKey, PropertyStorage>,
    pub own_constants: IndexMap<ConstKey, Union>,
    pub template_params: Vec<TemplateParam>,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_readonly: bool,
    pub location: Span,
}

pub struct MethodStorage {
    pub name: Arc<str>,
    pub params: Vec<FnParam>,
    pub return_type: Option<Union>,          // from annotation
    pub inferred_return_type: Option<Union>, // from analysis
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub template_params: Vec<TemplateParam>,
    pub assertions: Vec<Assertion>,
    pub location: Span,
}

pub struct FnParam {
    pub name: Arc<str>,
    pub type_: Option<Union>,
    pub default: Option<Union>,  // type of default value
    pub is_variadic: bool,
    pub is_byref: bool,
    pub is_optional: bool,
}
```

#### Key Methods

- [x] `get_method(fqcn, method_name)` — walks inheritance chain through classes, interfaces, enums
- [x] `get_property(fqcn, prop_name)` — walks inheritance chain
- [x] `extends_or_implements(child, ancestor) -> bool`
- [x] `type_exists(fqcn) -> bool` — checks classes, interfaces, traits, enums
- [x] `resolve_class_name(file, name) -> String` — resolves short names via `use` aliases + namespace
- [x] `mark_method_referenced` / `mark_property_referenced` / `mark_function_referenced` (M18)
- [ ] `get_all_methods(fqcn)` — own + inherited iterator

#### Finalization Pass

- [x] Build full ancestor chains (`all_parents`) for all classes
- [ ] Detect circular inheritance (emit error, break cycle)
- [x] Resolve trait use: copy trait methods into using class
- [x] Build complete method dispatch tables (own methods override inherited)
- [x] Validate: all `implements` interfaces are actually fulfilled
- [x] Validate: all abstract methods are implemented in concrete classes

**Exit criteria:** Resolve method on a class with 5 levels of inheritance correctly. ✅

---

### M5 — Pass 1: Definition Collection ✅
**Goal:** Walk all files and populate the codebase without analyzing bodies.

- [x] `DefinitionCollector` — AST visitor that extracts:
  - Class / interface / trait / enum declarations with full member signatures
  - Function declarations with params and return types
  - Top-level constants
  - `use` / `namespace` statements — populates `Codebase::file_imports` and `file_namespaces`
- [x] Run sequentially (parallel causes DashMap contention on small projects; rayon used in Pass 2)
- [x] Writes directly into `Codebase` (no fragment merge needed with DashMap)
- [ ] Handle duplicate definitions: emit `DuplicateClass` / `DuplicateFunction` issue
- [ ] Handle `require` / `include` — record dependency edge (don't follow inline)

**Exit criteria:** After pass 1 on a 100-file project, all classes and functions are in the codebase. ✅

---

### M6 — Issue System (`mir-issues`)
**Goal:** Typed issue kinds with severity, suppression, and output formatting.

#### Issue Kinds (initial set)

```
Undefined:
  UndefinedVariable, UndefinedFunction, UndefinedMethod, UndefinedClass,
  UndefinedProperty, UndefinedConstant, UndefinedMagicMethod,
  PossiblyUndefinedVariable, PossiblyUndefinedArrayOffset

Nullability:
  NullArgument, NullPropertyFetch, NullArrayAccess, NullFunctionCall,
  PossiblyNullArgument, PossiblyNullPropertyFetch, PossiblyNullArrayAccess,
  NullableReturnStatement, FalsableReturnStatement

Type mismatches:
  InvalidReturnType, InvalidArgument, InvalidPropertyAssignment,
  InvalidScalarArgument, InvalidCast, InvalidOperand,
  MismatchingDocblockReturnType, MismatchingDocblockParamType

Array issues:
  InvalidArrayOffset, InvalidArrayAssignment, NonExistentArrayOffset,
  PossiblyInvalidArrayOffset, MixedArrayOffset, MixedArrayAssignment

Redundancy:
  RedundantCondition, RedundantCast, UnnecessaryVarAnnotation,
  TypeDoesNotContainType, ParadoxicalCondition, RedundantNullComparison

Dead code:
  UnusedVariable, UnusedParam, UnusedMethod, UnusedProperty,
  UnusedClass, UnusedImport, UnreachableCode

Inheritance:
  UnimplementedAbstractMethod, UnimplementedInterfaceMethod,
  MethodSignatureMismatch, PropertyTypeMismatch, OverriddenMethodAccess

Security (taint):
  TaintedInput, TaintedHtml, TaintedSql, TaintedShell,
  TaintedFile, TaintedHeader, TaintedCookie

Other:
  DeprecatedMethod, DeprecatedClass, DeprecatedProperty,
  InternalMethod, InternalClass, InvalidThrow, MissingThrowsDocblock,
  ParseError, InvalidDocblock
```

#### Issue Struct

```rust
pub struct Issue {
    pub kind: IssueKind,
    pub severity: Severity,   // Error | Warning | Info
    pub file: Arc<str>,
    pub line: u32,
    pub col_start: u16,
    pub col_end: u16,
    pub message: String,
    pub snippet: Option<String>,   // source line
    pub suppressed: bool,
}
```

#### Severity Levels (1–8)

| Level | Issues reported |
|-------|----------------|
| 1 | Errors only (undefined, invalid returns) |
| 2 | + Possibly-undefined |
| 3 | + Nullable issues |
| 4 | + Possibly-nullable |
| 5 | + Type mismatches on docblock types |
| 6 | + Deprecated usage |
| 7 | + All warnings |
| 8 | + Info / style issues |

#### Output Formats

- [x] Text (default): `file.php:42:5 error UndefinedVariable: $foo is not defined`
- [x] JSON: array of issue objects
- [x] GitHub Actions annotations: `::error file=...,line=...::`
- [ ] JUnit XML: for CI integration
- [ ] SARIF: for GitHub Code Scanning

#### Suppression

- [x] `@psalm-suppress IssueName` in docblock before a statement
- [x] `IssueBuffer::suppress_range` — suppress issues emitted within a statement

**Exit criteria:** Issues print with correct location; `@psalm-suppress` suppresses an issue. ✅

---

### M7 — Expression Analyzer ✅
**Goal:** Infer types for all PHP expression kinds.

#### Context (implemented)

```rust
pub struct Context {
    pub vars: IndexMap<String, Union>,
    pub assigned_vars: HashSet<String>,
    pub possibly_assigned_vars: HashSet<String>,
    pub self_fqcn: Option<Arc<str>>,
    pub parent_fqcn: Option<Arc<str>>,
    pub static_fqcn: Option<Arc<str>>,
    pub fn_return_type: Option<Union>,
    pub inside_loop: bool,
    pub inside_finally: bool,
    pub inside_constructor: bool,
    pub strict_types: bool,
    pub tainted_vars: HashSet<String>,   // M19
}
```

#### Expressions to Handle

- [x] `Variable` — lookup in context; emit `UndefinedVariable` / `PossiblyUndefinedVariable`
- [x] `Assign($x = expr)` — infer RHS, set LHS; taint propagation
- [x] `AssignOp($x += expr)` — all compound assignment operators
- [x] `BinaryOp` — arithmetic, comparison, logical, string concat, null coalescing, instanceof
- [x] `UnaryOp` — `!`, `-`, `~`, `++`, `--` (prefix and postfix)
- [x] `Scalar` — `TLiteralInt`, `TLiteralFloat`, `TLiteralString`, `TBool`, `TNull`
- [x] `Array([k => v, ...])` — infer `TArray` with key/value types
- [x] `ArrayAccess($arr[$key])` — return value type; null/false checks
- [x] `PropertyFetch($obj->prop)` — resolve property type; null/mixed checks; taint
- [x] `NullsafePropertyFetch($obj?->prop)` — strips null from receiver; result is nullable
- [x] `StaticPropertyFetch(Cls::$prop)` — mixed
- [x] `ClassConstFetch(Cls::CONST)` — `Foo::class` → `TClassString(Some(fqcn))`
- [x] `Instanceof($x instanceof Foo)` — returns `TBool`; narrowing via `narrow_from_condition`
- [x] `Cast((int)$x)` — returns target type
- [x] `Ternary($c ? $a : $b)` — fork context, narrow, infer both branches, merge
- [x] `NullCoalesce($a ?? $b)` — `remove_null(A) | B`
- [x] `Match(expr) { ... }` — arms forked; subject narrowed to arm type; contexts merged
- [x] `Closure(fn(...) { })` — full body analysis; captures `use` vars; infers return type
- [x] `ArrowFunction(fn(...) => expr)` — full body analysis; captures outer vars implicitly
- [ ] `FirstClassCallable(strlen(...))` — return `Closure` type
- [x] `New(ClassName(...))` — resolves FQCN via `use` imports; checks constructor args; returns `TNamedObject`
- [x] `Clone($obj)` — returns same type as `$obj`
- [ ] `Yield` / `YieldFrom` — generator analysis
- [x] `Print` / `Echo` — taint check; returns `TInt(1)` / `TVoid`
- [x] `Isset($x)` / `Empty($x)` — returns `TBool`; narrowing handled in `narrow_from_condition`
- [x] `List([$a, $b] = ...)` / Array destructuring — assigns value type from `TArray`/`TList`
- [x] `Throw` as expression (PHP 8)
- [ ] `Spread(...$arr)` — in function calls and array literals

**Exit criteria:** Infer correct type for 50 representative PHP snippets. ✅

---

### M8 — Statement Analyzer ✅
**Goal:** Walk statement nodes, threading context through control flow.

#### Statements to Handle

- [x] `Expression` — delegate to `ExpressionAnalyzer`
- [x] `Return(expr)` — check expr type vs declared return type; emit `InvalidReturnType`; store in `return_types`
- [x] `Throw(expr)` — validates thrown type extends `Throwable`; emits `InvalidThrow`
- [x] `Echo` / `Print` — taint check; analyze expressions
- [x] `If/ElseIf/Else` — fork context, narrow via `narrow_from_condition`, merge branches
- [x] `While(cond) { }` — narrow on entry; fixed-point widening over body
- [x] `Do { } while(cond)` — analyze body first, then condition
- [x] `For(init; cond; post) { }` — init, narrow, body+post, widen
- [x] `Foreach($arr as $k => $v) { }` — infers key/value types from `TArray`/`TList`
- [x] `Switch(expr) { case: }` — subject variable narrowed to literal type per case arm
- [x] `Match` (expression — handled in ExpressionAnalyzer)
- [x] `Break` / `Continue` — propagate loop scope
- [x] `Try/Catch/Finally` — exception var typed per catch clause; finally runs unconditionally
- [x] `Static($x = expr)` — treated as regular assignment
- [x] `Global($x)` — mark as mixed
- [x] `Unset($x)` — remove variable from context
- [x] `InlineHTML` — no-op
- [x] `Declare(strict_types=1)` — sets `ctx.strict_types`
- [x] `@psalm-suppress` scanning before each statement
- [ ] Anonymous class inline declarations

**Branch merging rules:**

```
After if/else:
  - var exists in both branches → merge union of types
  - var exists in if-only, no else → mark as possibly_undefined in outer
  - var assigned in both + same type → concrete assignment in outer

After loop:
  - vars modified in body → widen to union with pre-loop type
  - vars introduced in body → mark as possibly_undefined after loop
```

**Exit criteria:** Correctly track a variable through nested if/else/try/foreach. ✅

---

### M9 — Call Analyzer ✅
**Goal:** Resolve and type-check function and method calls.

#### Function Calls

- [x] Resolve function name — tries namespace-qualified name first, falls back to global
- [x] Look up `FunctionStorage` in codebase
- [x] Check argument count (too few: `TooFewArguments`, too many: `TooManyArguments`)
- [x] Match positional and named arguments to parameters (PHP 8 named args)
- [x] Check each arg type is a subtype of param type; emit `InvalidArgument`
- [x] Handle variadic params: collect remaining args into array
- [ ] Handle by-reference params: update variable type in caller context
- [x] Resolve return type (annotation > inferred > `TMixed`)
- [ ] Apply `@psalm-assert` annotations to caller context after the call
- [ ] Handle `@psalm-pure`: check for side effects in pure functions
- [x] ~170 built-in functions with hand-coded return types in `call.rs`

#### Method Calls

- [x] Resolve receiver type (`$obj->method()`)
- [x] Handle `TMixed` receiver → return `TMixed`, emit `MixedMethodCall`
- [x] Handle union receivers → check all types, merge return types
- [x] Handle nullable receiver → emit `NullMethodCall` / `PossiblyNullMethodCall`
- [x] Dispatch through inheritance via `Codebase::get_method`
- [ ] Handle `__call` magic method fallback
- [ ] Handle `@method` docblock annotations on class
- [x] Check visibility: `private` outside class, `protected` outside hierarchy

#### Static Calls

- [x] `ClassName::method()` — resolve class name via `use` imports then `resolve_static_class`
- [x] `self::` — resolved via `ctx.self_fqcn`
- [x] `parent::` — resolved via `ctx.parent_fqcn`
- [x] `static::` — resolved via `ctx.static_fqcn`

#### Constructor (`new ClassName()`)

- [x] Resolve class name via `use` imports + namespace
- [x] Find `__construct` method
- [x] Check args against constructor params
- [x] Return `TNamedObject{fqcn}`
- [x] Handle `new static()` / `new self()` via expression fallback
- [ ] Handle `new $variable()` — emit warning

**Exit criteria:** Catch a wrong argument type on a method call; resolve inherited methods. ✅

---

### M10 — Type Narrowing Analyzer ✅
**Goal:** Refine variable types based on conditions.

#### Narrowing Sources

- [x] `$x instanceof ClassName` → narrow to `TNamedObject{fqcn}` (class name resolved via imports)
- [x] `is_string($x)` → narrow to `TString` in true branch; remove `TString` in false branch
- [x] `is_int`, `is_float`, `is_bool`, `is_null`, `is_array`, `is_object`, `is_callable`
- [x] `$x === null` / `$x !== null` → add/remove `TNull`
- [x] `$x === true` / `$x === false` → narrow to `TTrue` / `TFalse`
- [ ] `$x === 'literal'` → narrow to `TLiteralString`
- [ ] `$x === 42` → narrow to `TLiteralInt`
- [x] `if ($x)` (truthy) → remove null/false/0/empty-string
- [x] `if (!$x)` (falsy) → keep only falsy atomics
- [x] `isset($x)` → remove `TNull`; mark as definitely assigned
- [x] `empty($x)` handled via truthy/falsy narrowing
- [x] `$x && $y` → narrow both in true branch
- [x] `$x || $y` → narrow both in false branch
- [x] `!($x instanceof Foo)` → inverse instanceof narrowing
- [ ] `assert(is_string($x))` → narrow after assert
- [ ] Functions annotated with `@psalm-assert` → apply assertion

#### Switch subject narrowing

- [x] `switch ($x) { case 'foo':` → `$x` narrowed to `TLiteralString("foo")` inside case body

#### Clause System

Not yet implemented (complex conditions use ad-hoc recursion instead).

**Exit criteria:** After `if ($x !== null && is_string($x))`, `$x` is `TString` (not `TString|null`). ✅

---

### M11 — Class Analyzer ✅
**Goal:** Validate class definitions, inheritance, and interface compliance.

- [x] Abstract method check: every `abstract` method in a parent/interface is implemented
- [x] Method signature compatibility: overriding method return type checked (covariant)
- [ ] Property type compatibility: overriding property types must be identical (invariant)
- [x] Constructor promotion: `public function __construct(public readonly string $name)`
- [ ] Interface constant check: interface constants are implemented
- [x] Readonly property enforcement: assignment outside constructor emits `ReadonlyPropertyAssignment`
- [x] Final class/method: emit error on override (`FinalClassExtended`, `FinalMethodOverridden`)
- [ ] Trait conflict resolution: handle `insteadof` and `as`
- [ ] Circular inheritance detection

**Exit criteria:** Detect a missing `implements` method; detect a covariant return type violation. ✅

---

### M12 — Loop Analysis ✅
**Goal:** Correctly track types across loops without infinite analysis.

**Algorithm (widening):**

```
1. Record pre-loop context (C0)
2. Analyze loop body with C0 → produces C1
3. Merge C0 and C1 → C2 (widen: union of types)
4. Analyze loop body again with C2 → produces C3
5. If C3 == C2: fixed point reached, continue after loop with C2
6. Else: widen again (C4 = merge(C2, C3)) — usually converges in 2-3 iterations
7. Cap iterations at 3; fall back to TMixed for non-converging vars
```

- [x] `analyze_loop` helper with fixed-point widening (capped at 3 iterations)
- [x] Handle `break` / `continue`
- [x] Handle `foreach($arr as $k => $v)` — key/value types from `TArray`/`TList`
- [ ] Handle `foreach` with reference: `foreach($arr as &$v)`
- [ ] Generator functions: `yield $v` / `yield $k => $v`

---

### M13 — Generic Types (`@template`) ✅
**Goal:** Support generic annotations (`@template`).

- [x] Parse `@template T` / `@template T of UpperBound` from class/function docblocks
- [x] Store `TemplateParam { name, bound }` on `ClassStorage` / `MethodStorage` / `FunctionStorage`
- [x] During call analysis: `infer_template_bindings` maps params to arg types
- [x] Substitute template params in return type: `Union::substitute_templates`
- [x] Emit `InvalidTemplateParam` if inferred type violates declared bound
- [ ] Handle `@extends ClassName<ConcreteType>` — bind template params at class level
- [ ] Handle `@implements InterfaceName<ConcreteType>`
- [ ] Propagate template params through method chains

**Exit criteria:** `array_map(fn(int): string, [1,2,3])` returns `list<string>`.

---

### M14 — Pass 2: Body Analysis Orchestration ✅
**Goal:** Run the full analyzer over every function/method body, in parallel.

- [x] Collect all analyzable units: top-level functions, methods (including those in namespaces)
- [x] For each unit, create a fresh `Context` seeded from param types via `Context::for_method`
- [x] Run `StatementsAnalyzer::analyze_stmts`
- [x] Collect issues from `IssueBuffer` per unit
- [x] Store inferred return type (`merge_return_types`) back onto `MethodStorage` / `FunctionStorage`
- [x] Run in parallel with `rayon::par_iter` per file; each file is independent
- [ ] Handle recursive functions: detect cycle, use declared return type or `TMixed`
- [ ] Cross-function inference ordering: topological sort by call graph

---

### M15 — Configuration (`mir-config`) ❌
**Goal:** Config file support for project-level tuning.

#### `mir.xml` Schema

```xml
<mir>
  <projectFiles>
    <directory name="src/" />
    <ignoreFiles>
      <directory name="vendor/" />
    </ignoreFiles>
  </projectFiles>

  <issueHandlers>
    <UndefinedVariable errorLevel="error" />
    <UnusedVariable errorLevel="suppress" />
    <MixedAssignment errorLevel="info" />
  </issueHandlers>

  <plugins>
    <pluginClass class="MyPlugin\TypePlugin" />
  </plugins>

  <stubs>
    <file name="stubs/custom.phpstub" />
  </stubs>

  <phpVersion>8.2</phpVersion>
  <errorLevel>3</errorLevel>
  <findUnusedCode>true</findUnusedCode>
  <findUnusedVariables>true</findUnusedVariables>
  <checkForThrowsDocblock>false</checkForThrowsDocblock>
  <strictBinaryOperands>false</strictBinaryOperands>
  <rememberPropertyAssignments>true</rememberPropertyAssignments>
</mir>
```

- [ ] Parse `mir.xml` with `quick-xml`
- [ ] Support per-issue-kind error level overrides
- [ ] Support baseline file: `mir --set-baseline` generates `mir-baseline.xml`; subsequent runs
      suppress issues present in the baseline
- [ ] `@psalm-suppress` / `@mir-suppress` in docblocks — suppress per-site
- [ ] `// @psalm-suppress IssueName` inline comment suppression

---

### M16 — CLI (`mir-cli`) ⚠️ Partial
**Goal:** Polished command-line interface.

```
USAGE:
    mir [OPTIONS] [PATHS]...

ARGS:
    [PATHS]    Files or directories to analyze (overrides config)

OPTIONS:
    -c, --config <FILE>        Config file [default: mir.xml]
    --format <FORMAT>          Output format: text|json|github|junit|sarif [default: text]
    --error-level <1-8>        Override error level
    -j, --threads <N>          Parallelism [default: logical CPUs]
    --no-cache                 Disable cache
    --clear-cache              Delete cache and exit
    --set-baseline             Write current issues to baseline file
    --update-baseline          Add new issues to existing baseline
    --show-info                Include info-level issues in output
    --find-dead-code           Enable dead code detection
    --php-version <X.Y>        Override PHP version
    --no-progress              Disable progress bar
    --stats                    Print analysis statistics
    -q, --quiet                Suppress all output except errors
    -v, --verbose              Verbose output
    --version                  Print version
```

**Implemented flags:** `--format text|json|github|junit|sarif`, `--cache-dir`, `--show-info`,
`--stats`, `--quiet`, `--verbose`, `--threads`, `--php-version`, `--no-progress`, `[PATHS]`

- [x] Progress bar with `indicatif` (files analyzed / total); suppressed by `--no-progress`, `--quiet`, non-text formats
- [x] `on_file_done` callback on `ProjectAnalyzer` drives progress from inside the rayon loop
- [x] Colored output with `owo-colors` (respects `NO_COLOR`)
- [x] Exit code: `0` = no issues, `1` = issues found
- [x] `--stats`: files analyzed, error/warning counts, wall-clock time
- [x] `--verbose`: per-file issue counts after the run
- [x] `--threads` / `--php-version` / `--no-progress` / `--quiet`
- [x] JUnit XML output (`--format junit`) — grouped by file, CI-compatible
- [x] SARIF 2.1.0 output (`--format sarif`) — GitHub Code Scanning compatible, with rule definitions
- [ ] `--set-baseline` / `--update-baseline` (needs M15 — Configuration)
- [ ] `--no-cache` / `--clear-cache`
- [ ] `--error-level <1-8>` override
- [ ] `--find-dead-code` flag (dead code currently always runs)

---

### M17 — Cache Layer (`mir-cache`) ✅
**Goal:** Incremental analysis: skip unchanged files on re-runs.

**Implemented:** `mir-cache` crate with SHA-256 content hashing, JSON-backed per-file issue
cache, `AnalysisCache::open(dir)`, `get(file, hash)`, `put(file, hash, issues)`, `flush()`.
`--cache-dir <DIR>` flag enables it in the CLI. Pass 2 skips re-analysis on cache hits.

- [x] For each file: compute SHA-256 hash; if matches cache, return stored issues
- [x] Cache miss: analyze and store result
- [x] `flush()` writes all dirty entries to `{cache_dir}/cache.json`
- [ ] Cache invalidation across files (reverse dependency graph)
- [ ] `bincode` serialization (currently JSON)
- [ ] Cache versioning keyed by `mir` version

**Exit criteria:** Second run on unchanged project is >10x faster than first run.

---

### M18 — Dead Code Detection ✅
**Goal:** Find unreferenced classes, methods, functions, variables.

**Implemented:** `DeadCodeAnalyzer` runs after Pass 2. `Codebase` accumulates references in
`DashSet` during analysis. Dead code check inspects `private` methods and properties only.

- [x] Reference tracking during Pass 2: `mark_method_referenced`, `mark_property_referenced`,
      `mark_function_referenced` called from call/expr analyzers
- [x] `DeadCodeAnalyzer::analyze()` — walks all classes, checks private own_methods/own_properties
- [x] `UnusedMethod`, `UnusedProperty` at `Info` severity
- [x] Magic methods skipped (always considered reachable)
- [ ] `UnusedClass`, `UnusedVariable`, `UnusedParam`
- [ ] Respect `@psalm-api` / `@api` annotation
- [ ] Public method dead code (requires entry-point config)

---

### M19 — Taint Analysis (Data Flow) ✅
**Goal:** Track data from untrusted sources to dangerous sinks for security analysis.

**Implemented:** Inline taint tracking via `Context::tainted_vars` (`HashSet<String>`).
Simpler than a full data-flow graph but catches direct taint paths.

#### Model (implemented)

```
Sources: $_GET, $_POST, $_COOKIE, $_REQUEST, $_FILES, $_SERVER, $_ENV
         (seeded when accessing these superglobals)

Sinks:   echo/print → TaintedHtml
         mysql_query/pg_query/mysqli_* → TaintedSql
         exec/shell_exec/system/passthru/proc_open → TaintedShell

Sanitizers: htmlspecialchars, htmlentities, intval, strip_tags (break taint)
```

- [x] `Context::tainted_vars` — propagated through `=` assignments
- [x] Taint union in `merge_branches` (conservative: if either branch taints, result is tainted)
- [x] Sink check in `analyze_function_call` before arg evaluation
- [x] `is_expr_tainted` — recursive check for taint in sub-expressions
- [ ] `DataFlowGraph` — proper inter-procedural taint tracking
- [ ] `@psalm-taint-source` / `@psalm-taint-sink` / `@psalm-taint-escape` annotations
- [ ] Path traversal, header injection sinks

---

### M20 — Plugin System ❌
**Goal:** Allow custom rules and type resolvers via a stable plugin API.

```rust
pub trait Plugin: Send + Sync {
    fn get_name(&self) -> &str;

    // Called for every expression — return None to not override type
    fn get_expression_type(
        &self,
        expr: &Expr,
        context: &Context,
        codebase: &Codebase,
    ) -> Option<Union> { None }

    // Called after a function call is analyzed
    fn after_function_call(
        &self,
        call: &FunctionCallExpr,
        return_type: &mut Union,
        context: &mut Context,
        codebase: &Codebase,
        issues: &mut Vec<Issue>,
    ) {}

    // Called for each custom issue check
    fn check_statement(
        &self,
        stmt: &Stmt,
        context: &Context,
        codebase: &Codebase,
        issues: &mut Vec<Issue>,
    ) {}
}
```

- [ ] Plugin registration via config
- [ ] Dynamic loading via `libloading` (shared library `.so` / `.dylib`) OR
      static registration for built-in plugins
- [ ] Provide a `PluginContext` with safe access to codebase, config, issue emission

---

## Dependency Map

```
mir-ast
  └── mir-parser
        ├── mir-types
        │     └── mir-codebase
        │           └── mir-stubs
        │                 └── mir-analyzer
        │                       ├── mir-issues
        │                       └── mir-cache
        │                             └── mir-cli
        └── mir-config
```

---

## Crate Dependencies

```toml
[workspace.dependencies]
# PHP parsing
php-parser-rs = "0.1"

# Parallelism
rayon    = "1"
dashmap  = "6"

# Data structures
indexmap = "2"
smallvec = { version = "1", features = ["union"] }

# Serialization (cache)
serde    = { version = "1", features = ["derive"] }
bincode  = "2"
sha2     = "0.10"

# Config
quick-xml = "0.36"
toml      = "0.8"

# CLI
clap        = { version = "4", features = ["derive"] }
indicatif   = "0.17"
owo-colors  = "4"

# Diagnostics output
miette = { version = "7", features = ["fancy"] }

# Embedding stubs
include_dir = "0.7"

# Error handling
thiserror = "2"
anyhow    = "1"
```

---

## Known Hard Problems

These require careful design upfront; retrofitting is costly.

| Problem | Mitigation |
|---------|-----------|
| Generic type inference (bidirectional) | Implement Hindley-Milner style unification for template params |
| `$this` type narrowing in conditionals | Track `$this` as a special var in context |
| Recursive function return type | Use declared annotation; fall back to `TMixed`; warn if no annotation |
| Trait conflict resolution | Resolve at codebase finalization, before analysis |
| Magic methods (`__get`, `__call`) | Check for `@method` / `@property` docblocks; fall back to `TMixed` |
| Circular class dependencies | Detect during finalization; break cycle; emit error |
| First-class callables (`strlen(...)`) | Infer `Closure(string): int` from function storage |
| Named arguments (PHP 8) | Match by name before type-checking; reorder to positional |
| Match exhaustiveness | Require `default` arm or prove all cases covered from type |
| Cache invalidation across files | Reverse dep graph; invalidate transitively on class change |
| `extract()` / `compact()` | Emit `TMixed` for all vars in scope; warn |
| Variable variables (`$$x`) | Emit `TMixed`; warn unconditionally |
| `eval()` | Treat as `TMixed` returning; warn unconditionally |
| Dynamic `include`/`require` | Cannot follow; treat return as `TMixed` |

---

## Testing Strategy

| Layer | Approach |
|-------|---------|
| Type system | 500+ unit tests: subtype, merge, narrowing operations |
| Parser | Snapshot tests: PHP source → expected AST |
| Stubs | Integration: every stub function resolves without error |
| Analyzer | Fixture tests: `.php` file + expected issues (kind, line, message) |
| End-to-end | Run against known open-source PHP projects; compare issue counts |
| Performance | Benchmark: lines/sec on a 100k LOC project; must improve monotonically |
| Parity tests | Run same fixtures against reference analyzers; verify mir finds the same core issues |

#### Fixture test format

```
tests/fixtures/undefined_variable/
    input.php
    expected.json       # [{kind, line, message}, ...]
```

---

## Performance Targets

| Metric | Target |
|--------|--------|
| Cold analysis (100k LOC) | < 10 seconds on 8-core machine |
| Warm analysis (unchanged) | < 0.5 seconds (cache) |
| Memory usage (100k LOC) | < 512 MB |
| Throughput | > 50k LOC/sec (cold, parallel) |

Traditional PHP static analyzers typically process ~5–10k LOC/sec. A 5–10x speedup is realistic.

---

## Milestone Summary & Order

| # | Milestone | Crate(s) | Status |
|---|-----------|----------|--------|
| M0 | Workspace bootstrap | all | [x] |
| M1 | Type system | `mir-types` | [x] |
| M2 | Parser wrapper | `mir-parser` | [x] |
| M3 | Stubs | `mir-stubs` | [x] built into `mir-parser` |
| M4 | Codebase registry | `mir-codebase` | [x] |
| M5 | Pass 1: definition collection | `mir-analyzer` | [x] |
| M6 | Issue system | `mir-issues` | [x] |
| M7 | Expression analyzer | `mir-analyzer` | [x] |
| M8 | Statement analyzer | `mir-analyzer` | [x] |
| M9 | Call analyzer | `mir-analyzer` | [x] |
| M10 | Type narrowing | `mir-analyzer` | [x] |
| M11 | Class analyzer | `mir-analyzer` | [x] |
| M12 | Loop analysis | `mir-analyzer` | [x] |
| M13 | Generic types | `mir-types`, `mir-analyzer` | [x] |
| M14 | Pass 2 orchestration | `mir-analyzer` | [x] |
| M15 | Configuration | `mir-config` | [x] basic CLI config |
| M16 | CLI | `mir-cli` | [x] |
| M17 | Cache layer | `mir-cache` | [x] |
| M18 | Dead code detection | `mir-analyzer` | [x] |
| M19 | Taint analysis | `mir-analyzer` | [x] |
| M20 | Plugin system | `mir-analyzer` | [ ] |

**MVP** (useful analyzer): M0 through M16.
**Feature complete**: M0 through M20.

---

## Implementation Notes (updated as built)

### Actual crate structure
- `mir-types` — `Atomic`, `Union`, display; no external deps except `smallvec`/`serde`
- `mir-issues` — `IssueKind`, `Issue`, `IssueBuffer`, `Severity`, `Location`
- `mir-codebase` — `Codebase` (DashMap), all `*Storage` types, finalization
- `mir-parser` — wraps `php-rs-parser` 0.2.1 (arena-allocated, `parse(&Bump, src)`); `DocblockParser`; `type_from_hint`
- `mir-analyzer` — `DefinitionCollector` (Pass 1), `Context`, `ExpressionAnalyzer`, `StatementsAnalyzer`, `CallAnalyzer`, `narrowing`, `ProjectAnalyzer` (Pass 2)
- `mir-cli` — `clap` binary; text/json/github-actions output

### php-rs-parser API notes
- `parse(&arena: &Bump, src: &str) -> ParseResult` — two-arg, arena-first
- AST nodes carry `'arena` and `'src` lifetimes throughout
- `Visitor` trait in `php_ast::visitor` with default walk impls for all node types
- `Span { start: u32, end: u32 }` — byte offsets, not line/col; convert with `span_to_line_col`
- `Name::to_string_repr()` — returns `Cow<str>`, zero-alloc for simple names

### Design decisions taken
- Codebase stores **owned** data (`String`/`Vec`) — no arena lifetimes escape Pass 1
- `Union` uses `SmallVec<[Atomic; 2]>` with inline storage for the common 1–2 type case
- `Arc<str>` used for interned strings throughout; requires `serde` `rc` feature
- `IssueLocation` uses byte-offset→line/col conversion at issue creation time
- `ProjectAnalyzer::analyze` is sequential in Pass 1 (DashMap handles concurrent writes,
  but sequential avoids contention on small projects); Pass 2 will be parallel via rayon
