# Psalm-replacement readiness audit (app-server, 2026-08-27)

## STATUS: research only, nothing landed. Ran the release binary (rebuilt at HEAD `afd0bf68`)
against `~/dev/app-server` (real Frontify legacy app, 12,678 files, Psalm 6.16.1-configured,
`errorLevel="3"`, 1337-issue `psalm-baseline.xml`). Goal: what specifically blocks dropping mir
in as a Psalm replacement there, as opposed to the general false-positive hunting the sectors
below already do. Two categories of blocker turned up: **hard failures** (mir can't even run
against this project's real config) and **an architecture gap** (mir's error-level/severity
model doesn't track Psalm's own per-kind defaults, which inflates "new" findings far more than
any individual false positive does). Individual inference bugs are folded into the existing
sector lettering below where they match; genuinely new ones are logged as Sector N.

## P0 — mir cannot run at all against app-server's real `psalm.xml`

**Sector 0 reconfirmed with a concrete repro, and it's worse than "missing a feature" — it's a
hard abort.** app-server keeps Psalm in an isolated `vendor-tools/psalm/vendor/` (composer
`vendor-bin`-style split), not the project's main `vendor/`, and declares a custom plugin
(`tools/psalm/ConfigRegistryHooks.php`, implements `MethodReturnTypeProviderInterface` +
`AfterFunctionLikeAnalysisInterface`) via `<plugin filename="...">`. Running
`mir --stats --no-progress --format json .` from app-server's root against its own `psalm.xml`
exits 2 immediately with zero diagnostics:
```
mir: psalm plugin bridge: psalm plugin host: Psalm is not installed in this project — psalm
plugins need vimeo/psalm (it ships the plugin API the plugins are compiled against)
```
Root cause confirmed by reading `crates/mir-plugin/src/psalm/host.php:195` and
`crates/mir-plugin/src/psalm/mod.rs`: the host's `init` RPC handler already accepts an
`autoload` param (`$params['autoload'] ?? ($root . '/vendor/autoload.php')`), but
`BridgeOptions` (`mod.rs`) has no `autoload` field at all and `mir-cli/src/plugins.rs::setup_plugins`
never sends one — exactly the gap the existing Sector 0 carryover entry already predicted
("autoload RPC param exists in the PHP host but is never sent from Rust. Config knob +
wire-through."), now pinned against a real project instead of a hypothesis. And this isn't a
graceful degrade: `plugins.rs`'s own doc comment says a configured-but-failing plugin is a
**deliberate hard error (exit 2)**, "matching how Psalm treats broken plugins." Fix needs both
halves: (1) a config knob (e.g. a `<plugins autoload="...">` attribute or per-`<pluginClass>`
attribute) parsed in `mir-cli/src/config.rs`, (2) threading it through `BridgeOptions.autoload`
into the `init` call. Until then, the only workaround is stripping `<plugins>` from the config
before pointing mir at this project — which silently drops real, baselined signal (see P1).

**P0b — even with the autoload gap fixed, the bridge doesn't support this plugin's dominant
hook.** `ConfigRegistryHooks::afterStatementAnalysis` implements
`AfterFunctionLikeAnalysisInterface` to check `#[Config('key')]` parameter attributes against a
type registry (emits `ConfigTypeMismatch`). Per the psalm-bridge's own module doc
(`crates/mir-plugin/src/psalm/mod.rs:17-19`), only `addStubFile` and
`Function/MethodReturnTypeProviderInterface` are supported — "other hook registrations
(`AfterExpressionAnalysis`, taint hooks, …) are reported in warnings and skipped." So the
`MethodReturnTypeProviderInterface` half (`GetConfig::byKey()`/`GetConfigByKey::getOrFail()`
narrowing to `Option<T>`/`T`) would work once P0 is fixed, but the attribute-mismatch check
never runs — a real, currently-enforced check (`ConfigTypeMismatch`/`ConfigKeyNotRegistered`
are both in the 1337-issue baseline, 22 occurrences total) silently goes unchecked with no
warning that it's gone, since the "unsupported hook" warning only fires when the bridge
actually loads (blocked by P0 today).

### Design: P0 (autoload wiring) — IMPLEMENTED

Mechanical, single-process fix. `host.php`'s `init` already resolves `autoload` per-call, and
its one `require $autoload` covers every plugin in the process — so this is a **project-wide**
knob, not per-`pluginClass` (a per-plugin autoload would need one host subprocess per plugin,
not worth it for a case that doesn't exist yet).

Landed as a **dedicated `<psalmBridge autoload="...">` config element, not an attribute on the
shared `<plugins>` element.** `<plugins>` is Psalm's own config schema (`<pluginClass>`) plus
mir's `<rustPlugin>` extension — grafting a psalm-bridge-only setting onto it would mean an
attribute that's meaningless for `<rustPlugin>` entries and invisible as anything psalm-specific.
Concretely:

1. `mir-cli/src/config.rs`: new `PsalmBridgeConfig { plugins: Vec<PsalmPluginEntry>, autoload:
   Option<String> }`, replacing the old flat `Config.psalm_plugins`/`psalm_autoload` fields with a
   single `Config.psalm_bridge: PsalmBridgeConfig` — named and grouped so it reads as "the psalm
   bridge's config," and has an obvious home for future bridge-only settings (php binary override,
   timeouts, …) beyond `autoload`. `autoload` is captured from a new `<psalmBridge autoload="...">`
   element (root-level, own start/empty-tag handling, same pattern as `phpVersion` on the root).
   `<plugins>` parsing is unchanged, just repointed at `config.psalm_bridge.plugins`.
2. `mir-plugin/src/psalm/mod.rs`: `BridgeOptions.autoload: Option<PathBuf>` (defaults `None`,
   preserving today's `vendor/autoload.php` default in `host.php`). Included in the `init` RPC
   params only when set, mirroring how `php_binary` is already overridden from `plugins.rs`
   post-construction. Module doc now states the Rust-first positioning explicitly (see below).
3. `mir-cli/src/plugins.rs::setup_plugins`: resolve `config.psalm_bridge.autoload` against
   `config_base` and set `options.autoload` before `PsalmBridgePlugin::spawn`.
4. `host.php` needs **no change** — `$params['autoload'] ?? ($root . '/vendor/autoload.php')` is
   already there.

**Rust plugins are the recommended way to write new mir plugins**, not the Psalm bridge — the
bridge is a migration/compatibility path for projects that already have Psalm plugins. Documented
on `mir-plugin/src/psalm/mod.rs`'s module doc and `mir-cli/src/plugins.rs`'s doc comment, and on
the new `PsalmBridgeConfig` struct itself, so this doesn't drift back into looking like the primary
extension mechanism.

Tests: `config.rs` — parsing `<psalmBridge autoload="...">` (both an explicit close tag and
self-closed), no-`<psalmBridge>` leaving it `None`, and a same-named `autoload` attribute on the
generic `<plugins>` element being correctly ignored (proves the two schemas stay independent).
`psalm_bridge.rs` — a fixture project (`fake-psalm-project-isolated-vendor/`) laid out like
app-server (no `vendor/autoload.php` at the root, only a nested `vendor-tools/psalm/vendor/`
autoloader reaching into a sibling fixture's fake Psalm/plugin classes): one regression test
confirming the original hard-abort ("no composer autoloader") still fires without an override, one
proving `BridgeOptions.autoload` fixes it end-to-end (stub registration + a live return-type RPC
round trip).

### `<plugin filename="...">` support — IMPLEMENTED (found live-testing P0 against app-server)

Landing P0 alone against app-server's real `psalm.xml` surfaced a second, previously-unnoticed
parsing gap: app-server declares its plugin two ways —
`<pluginClass class="Psalm\PhpUnitPlugin\Plugin"/>` **and**
`<plugin filename="tools/psalm/ConfigRegistryHooks.php"/>`. `config.rs` only recognized
`<pluginClass>`; `<plugin filename>` (Psalm's *other* plugin syntax — the file is required
directly and Psalm registers its own first declared class as a hook, with no
`PluginEntryPointInterface`/`RegistrationInterface` in between) was silently dropped — not even a
warning, since the whole element was unrecognized. `ConfigRegistryHooks` — the subject of P0b —
never got a chance to even warn about its unsupported hook.

Fixed end-to-end:
- `PsalmPluginEntry` (config.rs) and `PsalmPluginSpec` (mod.rs) became enums —
  `Class { class, config_xml }` / `File { path }` — so both of Psalm's plugin syntaxes are
  first-class, not one bolted on as an optional field.
- `host.php`: a `kind: "file"` RPC entry requires the path directly. Finding *which* class to
  register as the hook can't use `get_declared_classes()` diffing (the file's own
  `require_once`d siblings would pollute the diff, and app-server's real plugin does exactly
  this — sibling classes required at the top, hook class declared after). Instead uses PHP's
  `token_get_all()` to statically find the first class *this file itself* declares — no
  PhpParser dependency for this narrow task, so no fake-PhpParser fixture needed to test it.
- Cross-file regression test (`bridge_supports_file_based_plugin_registration` in
  `psalm_bridge.rs`) mirrors this exact shape: `plugin/FileBasedHooks.php` `require_once`s a
  sibling file declaring an unrelated class *before* declaring its own hook class.

**Second bug found in the same pass, in code that predates this session and was assumed
already fully working:** testing this plugin's `MethodReturnTypeProviderInterface` half against
*real* Psalm 6.16.1 (not the test fixture's fake, simplified event classes) threw "cannot supply
constructor parameter `$stmt`" — real Psalm's `FunctionReturnTypeProviderEvent`/
`MethodReturnTypeProviderEvent` constructors take an actual `FuncCall`/`MethodCall|StaticCall`
node as `$stmt`, and `getCallArgs()` delegates to it; there's no separate `call_args` param the
old code's `'call_args' => $callArgs` mapping could satisfy. This was **invisible to the existing
test suite** because the fixture's fake event classes had an outdated shape (`array $call_args`
directly) — exactly the kind of drift a hand-rolled fake can hide. Fixed by having `host.php`
build (or reuse, when the re-parsed call snippet already matches) a real `FuncCall`/`MethodCall`
node for `$stmt`; fixture event classes updated to match real Psalm's actual constructor shape,
so the existing tests now actually guard this.

**A third, narrower gap, also fixed:** plugin code that reaches for `Psalm\Config::getInstance()`
(this plugin's use of `Psalm\Internal\Type\Comparator\UnionTypeComparator` does, transitively)
threw "No config initialized", since the bridge never constructs a real `Psalm\Config` (its
constructor is `protected`). Fixed by having `host.php` seed the singleton via Psalm's own
`Config::locateConfigFile()`/`loadFromXMLFile()` against the project's real config — and
separately, `PSALM_VERSION` (needed by `loadFromXMLFile`, normally defined by Psalm's CLI
bootstrap, which the bridge never runs) is now defined defensively from
`Psalm\Internal\VersionUtils`. Both no-ops (not warnings) when the project has no locatable
config or the class doesn't exist, so plain Rust-plugin-only projects see no new noise.

**Confirmed architectural floor, not fixed (correctly out of scope):** even with all of the
above, `ConfigRegistryHooks::getMethodReturnType()` still fails, now with "Typed static property
`Psalm\Internal\Analyzer\ProjectAnalyzer::$instance` must not be accessed before initialization"
— `UnionTypeComparator` reaches for a live, fully-initialized Psalm analysis session (real
`Codebase`, wired-up providers, the works), which is exactly the "clone Psalm's analyzer state
machine" wall the bridge's module doc already rules out (see P0b design below). Degrades
gracefully: the RPC call fails inside `host.php`'s own try/catch and returns `{"type": null}` as a
normal result (not an RPC-level error), so the bridge doesn't disable itself — it just silently
contributes no type for that provider call, identical to today's behavior, confirmed by an
unchanged 26,881-error/1,178-warning count across every run in this pass.

### Design: P0b (`AfterFunctionLikeAnalysisInterface`)

**Full generality is a non-goal.** The bridge's whole v1 model works because return-type
providers are *pulled* per call-site and `host.php` reconstructs just enough synthetic AST for
that one call (see `buildCallArgs`) — it never needs a real, fully-populated Psalm `Context`.
Hooks that inspect a function *body* (`AfterExpressionAnalysisInterface` reading arbitrary local
variable types mid-body, taint-flow hooks, anything keyed off Psalm's own data-flow graph) would
require cloning Psalm's analyzer state machine to feed them, which is what the module doc at
`mod.rs:17-19` already rules out. That stays unsupported, and the existing `registerHooks`
fallback (`mod.rs`/`host.php` line ~273: "`$class` registers `$short` — not supported ... yet,
skipped") already surfaces it as a visible warning once P0 lands — so P0b is a tracked
enhancement, not a silent-failure risk blocking P0.

**What `ConfigRegistryHooks` (and realistically most `AfterFunctionLikeAnalysisInterface`
plugins doing attribute/signature checks) actually touch is narrower than "full body analysis":
just the declaration surface** — `$event->getStmt()`'s params (name, type, `#[Attribute]`s) and
return type, plus enough location/class context to build a message and call
`IssueBuffer::maybeAdd`. mir already computes exactly this during declaration analysis
(`mir-analyzer/src/attributes.rs` already resolves attribute constructor args for
`InvalidAttribute` checks; `stmt/declarations.rs::analyze_function_decl_stmt` and
`body_analysis/classes.rs::analyze_method_scope` are the two declaration-analysis exit points).
Proposed scope, a "declaration-shape" tier:

1. **New native hook** in `mir-plugin/src/lib.rs`, following the exact `after_statement_analysis`
   pattern (`HookFlags::after_function_like_analysis`, zero-cost when unset): a
   `AfterFunctionLikeAnalysisEvent { name, is_method, class_fqcn, params: &[ParamInfo], return_type,
   file, issues: Vec<PluginIssue> }` where `ParamInfo { name, ty, attributes: &[AttributeInfo] }`
   reuses whatever attribute-args shape `attributes.rs` already resolves — no new inference, just
   exposing data mir already has.
2. **Fire sites**: end of `analyze_function_decl_stmt` and `analyze_method_scope`, gated on
   `plugins.hooks().after_function_like_analysis` like every other hook. Cheap local pre-filter:
   skip the call entirely for declarations with zero parameter attributes, since a hook whose only
   visible surface is attributes can't fire on one that has none.
3. **Bridge side**: `host.php`'s `registerHooks` grows a third bucket
   (`afterFunctionLikeHookClasses`) for `AfterFunctionLikeAnalysisInterface` registrations,
   returned from `init` like `functionIds`/`methodClasses` today; `mod.rs` sets
   `HookFlags.after_function_like_analysis` when that list is non-empty. New RPC method
   `afterFunctionLikeAnalysis`, one call per qualifying declaration, params =
   `{name, isMethod, classFqcn, params:[{name,type,attributes:[{class,args}]}], returnType, file}`.
   `host.php` builds a synthetic `Stmt\ClassMethod`/`Stmt\Function_` (fake `Param`/`AttributeGroup`
   nodes carrying literal attribute args only — same trick as the existing call-snippet
   reconstruction, not a real parse), reuses `buildEvent`'s reflection-based, name-matched
   constructor call to build a real `AfterFunctionLikeAnalysisEvent`, and invokes
   `$class::afterStatementAnalysis($event)`.
4. **Issue capture is the one genuinely new mechanism needed.** Plugins report via
   `IssueBuffer::maybeAdd(new SomeIssue(...), $suppressed)`, which writes to Psalm's static
   `IssueBuffer` state, not a return value. Psalm exposes exactly this capture seam for embedders:
   `IssueBuffer::startRecording()` before the call, `IssueBuffer::clearRecordedIssues()` after,
   translating each returned `IssueData` into the RPC result as
   `{name: issueType, message, severity}` — same shape mir already has a home for via
   `IssueKind::PluginIssue { name, message }` (`mir-issues/src/lib.rs:795`, already wired to
   `@mir-suppress`/baseline matching by name). **Risk to verify before committing to this, not
   assumed**: confirm `startRecording`/`clearRecordedIssues` exist with these exact names on the
   Psalm version(s) actually in the wild (reflect on `IssueBuffer` the same defensive way
   `MirShimGenerator` already treats interface shape) — these are internal-ish static APIs that
   could rename across major Psalm versions, and the fallback if they don't exist should be a
   warning + skip, not a hard error.
5. Update the module doc (`mod.rs:10-19`) to describe three tiers once this lands: full support
   (stubs, return-type providers), best-effort declaration-shape support (this), and unsupported
   body-context hooks — so the doc keeps matching reality instead of drifting stale again.

Sequencing recommendation: ship P0 alone first — it already unblocks running mir on app-server at
all, plus the `MethodReturnTypeProviderInterface` half of this exact plugin. P0b is a real,
separately-landable follow-up (new hook end-to-end: event type, two fire sites, RPC method, AST
reconstruction, `IssueBuffer` capture, tests) — track it as its own item rather than folding it
into the P0 fix.

## P1 — severity/error-level architecture doesn't track Psalm's per-kind defaults (single
largest source of "new" noise — 23%+ of all unbaselined findings from one cause)

A plugin-free run (`<plugins>` stripped to unblock P0) against app-server's real `psalm.xml`
(`errorLevel="3"`) + its real `psalm-baseline.xml`: 12,678 files, 23.85s, **2524 unbaselined
diagnostics** (1352 Error + 1172 Warning) on top of the existing 1337-issue baseline. Confirmed
architecturally, not just by sampling: `config.error_level` (parsed from `errorLevel="N"`) is
consumed in exactly one place in the entire CLI —
`crates/mir-cli/src/report.rs:125: show_info = cli.show_info || config.error_level >= 7`. That's
it. Every `IssueKind` instead gets one fixed `default_severity()` (`crates/mir-issues/src/lib.rs:807`)
that never varies with the config's error level. Real Psalm's `errorLevel` is a per-issue-kind
threshold (each of its ~200 issue types has its own baked-in default level; the project's
configured `errorLevel` decides which kinds surface at all, and at what severity) — mir has no
equivalent table, so it reports every kind it has a Warning/Error default for, regardless of
what the project's `psalm.xml` would have Psalm itself report.

Concretely confirmed for **PropertyPossiblyUninitialized (588/2524 = 23% of the entire
unbaselined set)**: real Psalm 6.16.1 run directly against sample files in this project reports
**zero** issues on lines mir flags — Psalm found the same properties but as
`PropertyNotSetInConstructor` at **Info** severity, invisible at this project's error level and
absent from the baseline entirely (0 occurrences in 1337 baselined issues, confirmed by grep).
mir emits its equivalent unconditionally as Warning. Same pattern independently confirmed for
zero-baseline-occurrence kinds `MethodSignatureMismatch`, `ReadonlyPropertyRedeclarationMismatch`,
`PropertyTypeRedeclarationMismatch`, `ImpossibleIdenticalComparison`, `NonExistentArrayOffset`,
`UnhandledMatchCondition`, `UnusedVariable`, `UnusedForeachValue`, `NullArgument`,
`InvalidStringClass`, `InvalidPropertyAssignment` — all hardcoded to `Severity::Warning` at
`mir-issues/src/lib.rs` lines ~830-905, none present anywhere in the 1337-issue baseline. This
single gap means pointing mir at *any* existing tuned `psalm.xml` will report far more than
Psalm ever did there, and the excess is dominated by whatever kinds that project's error level
was tuned to hide — not by mir's own inference quality. Fix shape: give `IssueKind` a real
per-kind intrinsic level (mirroring Psalm's own table) and gate `default_severity()`/emission on
`config.error_level` the same way Psalm gates its own issue list, not just the info-visibility
toggle.

## P2 — baseline compatibility is only partial (raw "unbaselined" counts above are an
upper bound, not exact — both directions confirmed)

`Baseline` (`crates/mir-cli/src/config.rs:367`) matches on exactly `(file, kind-name,
code-snippet-string)`, consume-once. Two independent failure modes confirmed:
- **Kind-name aliasing beyond the already-known D8 pair.** `RgbaColor.php:30-31` (`int` into
  `int<1,255>`) is baselined by real Psalm as `PropertyTypeCoercion`; mir emits the equivalent
  as `InvalidPropertyAssignment` — never matches, so a legitimate, already-accepted issue
  surfaces as "new" on every run. Add to D8's aliasing list:
  `InvalidPropertyAssignment` ↔ `PropertyTypeCoercion` (int-range coercions).
- **Exact-snippet mismatch even when the kind name matches.** `PropertyOption.php` (line not
  re-verified — sample from triage, re-check before fixing): mir's captured snippet for a
  `NullableReturnStatement` is the whole `return $this->propertyId;` statement; Psalm's baselined
  `<code>` is just the expression `$this->propertyId`. Same kind, same file, same underlying
  issue — doesn't match, counted as "new" anyway. mir's snippet capture needs to match Psalm's
  convention (bare expression, not enclosing statement) for `return`-statement-shaped issues at
  minimum; unknown how many other kinds have the same statement-vs-expression mismatch without a
  dedicated pass comparing mir's snippet capture against Psalm's for every kind that can baseline.

**Net effect of P1+P2 together: the 2524-unbaselined-issue count is not a clean "false positive
count."** It's inflated by kinds Psalm's own config would never have surfaced (P1) and possibly
slightly inflated further by baseline entries that should have matched but didn't (P2) — the
sampling below (Sector N, and sector cross-references) still stands on its own merits since it
was verified by reading source, not by trusting the raw count.

## Sector N — new inference gaps found this pass (app-server-specific repros)

**N1 — `TIntersection` as the SUPERTYPE has no structural-array arm (mirror of the already-fixed
B6). CONFIRMED, ~112/746 (15%) of the InvalidArgument bucket by itself.** `subtype.rs` (both
mir-analyzer's codebase-aware checker and `mir-types::union::is_subtype_structural`) only has
`(TNamedObject, TIntersection)` / `(TIntersection, TIntersection)` arms. A `TKeyedArray`
(array-shape literal) checked against a `TIntersection` supertype — the standard Psalm idiom
`@psalm-type Context = array<string,mixed> & array{actor:...,target?:...,outcome:...}` for
"shape plus extra keys allowed" — has no matching arm, falls through to `false`. Repro:
`application/module/Support/Account/EventSubscriber/ForceTwoFactorAuthenticationEventSubscriber.php:26`
calling `SecurityLogger::info(context: [...])` against
`application/module/Common/Logger/Contract/SecurityLogger.php:10-15`'s `@psalm-type Context`;
every field is structurally valid, confirmed by hand against the event's real property types.
Recurs identically on every `SecurityLogger`-family call site (`info`/`warning`/`error`/…). B6's
own carryover note flagged this exact residual as unfound — now pinned.

**N2 — `readonly`-implementer vs get-only-hook-interface override check has no compatibility
arm. CONFIRMED, ~69/81 of the Readonly/PropertyTypeRedeclarationMismatch bucket (not the
`@template-implements` case below).** `class/overrides.rs:875-905` flags a mismatch whenever
`own_prop.has_native_readonly != parent_prop.has_native_readonly`. A parent **interface**
declaring a PHP 8.4 get-only hooked property (`public int $x { get; }` — hooks can't carry the
`readonly` keyword in an interface at all) always has `has_native_readonly == false`; an
implementing `final readonly class` (or a promoted-readonly-property class) has `true` — valid,
compatible PHP, but flagged anyway. Spans 10+ unrelated interface families in this codebase
(`ResizeStrategy`, `AssetSearchCommand`, `CatalogCell`/`CatalogCellInput`, `JobSchedulingError`,
`JobStatus`, `UsageInterface`, `FigmaSyncException`, `ItemInput`, `AssetChange`/
`ScopedAssetChange`, `FileConversionResult`) — this is the codebase's dominant DDD value-object
idiom, not a one-off. Repro: `application/module/Common/Job/Contract/Data/JobStatus/{JobStatus,Finished}.php:10-15`.
Sibling of the existing L17 hooked-property lineage, but on the override-checking side rather
than uninitialized-property tracking.

**N3 — `@template-implements` binding not substituted into an inherited PROPERTY's native type
before the invariant redeclaration check. CONFIRMED, 6/6 PropertyTypeRedeclarationMismatch +
6/75 of the Readonly bucket.** `Id` (`application/utility/Shared/Id/Id.php`): `@template-covariant
T of positive-int|non-empty-string`, `public int|string $value { get; }`. `FileId`
(`.../AssetDelivery/Core/Domain/Asset/Source/File/FileId.php`): `@template-implements
Id<non-empty-string>`, narrows to `public string $value;`. mir compares the child's `string`
against the parent's un-substituted `int|string` bound instead of the bound-with-`T=non-empty-string`
substitution, and flags `PropertyTypeRedeclarationMismatch`. Property analog of the existing
G3/L9 template-substitution-on-override lineage (which so far only covered methods); same
`Id`/`IntegerId`/`Uuid`/`UuidV7`/`AssetScopeId`/`UsageLinkId` family as N2 above but a distinct
root cause — fix separately, N2's fix won't touch this.

**N4 — cross-method `@psalm-assert`-established non-null property state not propagated through
an intermediate callee. CONFIRMED, ~70/108 of the NullableReturnStatement bucket, new — not
matched by anything in ROADMAP's E4/E5 (those are `instanceof`-narrowing-through-a-call, not
`@psalm-assert`-on-property).** `application/utility/Database/Database.php`: `private ?mysqli
$connection`, `connect()` is annotated `@psalm-assert mysqli $this->connection`. `insert()` calls
`bulkInsert()` (which itself unconditionally calls `$this->connect()`) and then reads
`$this->connection->insert_id` — never calling `connect()` directly itself. mir doesn't trust the
non-null state established transitively through the callee's own connect-then-use sequence, so
every caller of `insert()`/`lastId()`-family methods gets `int|string|null` back and flags
`NullableReturnStatement` downstream (confirmed clean in `psalm-baseline.xml` at all ~9 sampled
call sites — genuine mir gap, not real signal). Repros:
`application/module/AssetDelivery/Core/Infrastructure/Repository/AssetVariantRepository.php:237`,
`.../AttachmentRepository.php:484`, `.../PublishingJobRepository.php:83`.

**N5 — `use const` import of a PECL/native-extension namespaced constant unresolved. CONFIRMED,
14/16 of the UndefinedConstant bucket, new.** `application/utility/DtoRule.php`:
`use const ast\AST_CLASS;` / `use const ast\flags\CLASS_READONLY;` (the `ast` extension), then
used unqualified (`$node->kind === AST_CLASS`). Either the `ast` extension has no stub constants
at all, or `use const Ns\NAME;` import resolution doesn't reach constant lookup generally —
needs a follow-up repro isolating which.

**N6 — dynamic class-constant fetch through a `class-string` variable leaks an un-substituted
`self::`-relative name. CONFIRMED, 2/16 of the UndefinedConstant bucket, new.**
`application/module/.../SettingsSchemaProvider.php:103`: `is_string($className::SETTINGS_SCHEMA)
? ... : null` where `$className` is a `class-string` variable. mir reports the constant as the
garbled `Frontify\Model\Data\DocumentBlock\self::SETTINGS_SCHEMA` — a `self::` token from
`SETTINGS_SCHEMA`'s own declaration site leaking into the resolved name instead of being
rewritten to the receiver's actual FQCN.

## Cross-references confirmed/strengthened by this pass (no new investigation needed, just
volume/priority data for the existing entries)

- **D8** (`PropertyNotSetInConstructor` ↔ `PropertyPossiblyUninitialized` kind-name aliasing) —
  now shown to be entangled with P1 above and to explain the *entire* 588-instance
  PropertyPossiblyUninitialized bucket (8 samples, 100% Warning severity, 0% real signal found).
  This makes D8 + P1 together the single highest-leverage fix in the whole audit — 23%+ of all
  unbaselined findings from two related items.
- **J4** (bare generic instantiation rejected by a parameterized target) — now confirmed as the
  dominant cause (58/83 = 70%) of the InvalidPropertyAssignment bucket at production scale
  (`new CommandOption(...)`/`new CommandArgument(...)` into `Argument<T>`-typed properties, e.g.
  `application/cli/.../ExportPackage.php`, `OptInCommand.php`); none of the sampled sites are in
  the baseline.
- **L9** (un-restated override return vs parent's docblock-refined return) — confirmed as the
  dominant cause (≈75% of a 24-item sample, likely 150-200+/310 codebase-wide) of
  MethodSignatureMismatch, via one repeated shape:
  `protected static function getSingularClassName(): string` overriding an abstract/interface
  method documented `@return class-string<T>`, recurring across the entire
  `application/server/Model/Data/*Collection` hierarchy.
- **L8** (variadic param vs inherited docblock collected-array type) — confirmed via
  `GetAccessRequestService::byIds`, the exact shape already in ROADMAP's own headline repro.
- **L2** (class-constant wildcard docblock types never resolve) — confirmed recurring in
  argument-checking (not just comparisons) via `TargetObjectType::*`/`LastLoginMode::*`-style
  unions, several InvalidArgument/InvalidArrayOffset/ImpossibleIdenticalComparison/
  InvalidTemplateParam samples.
- **J5** (`callable(): void` rejects non-void-returning callables) and **J4/J10** (bare generic
  vs parameterized target) — both reconfirmed in the InvalidArgument sample.
- **F2** (`@psalm-import-type` alias never reaches resolution, cascades into InvalidOperand) —
  confirmed on `Pagination.php`'s `PageSize`/`PageNumber` aliases: unresolved alias names get
  treated as phantom classes in the current namespace, producing InvalidOperand,
  ImplicitToStringCast, and InvalidArrayOffset all from the same root cause.
- **C1-remnant/G12** (unbindable `class-string<T>` falls back to literal `T` → `UndefinedClass:
  T`) — confirmed as 9/9 of the UndefinedClass bucket.
- **ForbiddenCode** (`shell_exec`) — reconfirmed as genuine signal, same lineage as the harness's
  existing "Confirmed NOT bugs" entry.

## Not chased this pass (logged, not investigated)

InvalidReturnType (107) — heterogeneous, no single dominant cause found in sampling (test-file
array-shape mismatches, one `array_map`+`static()` generic-preservation gap on `Holders.php:256`
sibling to L29/L21, several `Frontify\Utility\Result<...>` generic-argument mismatches that read
as genuine app-server issues); needs its own dedicated pass if pursued. UnhandledMatchCondition
(42) and PossiblyUndefinedVariable (32) sampled but mostly read as real signal / legacy code, not
mir bugs — see triage notes, not independently re-verified beyond sampling. UnusedForeachValue
(16), InvalidStringClass (3, one Test file), InvalidCast (1), TooManyArguments (2), NullArgument
(1), InternalMethod (1) — volumes too low/scattered to classify without per-file work.

---
---

# Harness false-positive audit (packages 2-10)

## STATUS: active. Found via `harness/` (baseline diffs against 10 real Composer packages).
Each CONFIRMED item was reproduced standalone against the release binary before being logged here.
Fixed items land as individual commits with a `.phpt` regression test and are removed from this list
once landed — see `git log` for their history. `psr-log` swapped out for `phpunit-phpunit` (13.2.6) — its
baseline was empty (0 diagnostics, just PSR interfaces), no FP-hunting value. `psr-container`
(2.0.2) swapped out for `guzzlehttp/guzzle` (7.15.2) for the same reason (0 baseline diagnostics,
just PSR interfaces).

---

## Lower priority / deprioritized

- **P28 residual — a caller elsewhere using a function/method's return value
  still sees the unreconciled, docblock-shadowed-builtin return type.** P28's
  fix (`crate::util::reconcile_docblock_builtin_shadow`, applied at
  body-analysis flow-seeding for params/returns/methods; centralized in
  `find_property_in_chain` for both instance AND static properties, since
  they share that one query) covers every within-body member-lookup/
  flow-analysis consumer confirmed by repro, including `self::$prop`/
  `static::$prop`. NOT covered: a *caller* in another file/statement invoking
  a function/method and chaining a call on its return value — that reads
  `storage.return_type` via a separate call-resolution path
  (`call/function.rs`/`call/method.rs`), not body-analysis seeding. CONFIRMED
  repro: a method with `/** @return Generator */`/native `Generator` return
  (a same-namespace shadow), called from a second statement with `->build()`
  chained onto the result, still flags `UndefinedMethod`. Not yet found in
  the wild (same as P28 itself); revisit if it turns up in a future harness
  pass — likely needs the same `reconcile_docblock_builtin_shadow` call
  inserted wherever those two files resolve a callee's stored return type.

- **No divisibility narrowing after `$e -= $e % 2`.** `pow($aa, $e / 2)` on a value proven even by a
  preceding `% 2` + `-=` still yields `int|float` from `infer_div` (`expr/helpers.rs`) — would need
  tracking a "divisible-by-N" invariant through arithmetic. Real precision gap, harder fix, lower
  volume than the above. brick-math.

## Not yet investigated

- Every remaining kind in both **phpunit-phpunit** (882 baseline diagnostics) and **guzzlehttp-guzzle**
  (787) was audited to completion in the pass-3 sweep (Sector M) — 7 parallel audits, one per
  kind-family per package. Nothing left unsampled in either baseline as of this pass. Next new package
  added to `harness/packages.php` starts a fresh "not yet investigated" list here.

---

## Confirmed NOT bugs (legitimate signal — keep this list; don't re-investigate these)

- **`MethodSignatureMismatch` "must be re-declared @pure/@mutation-free when overridden"** (122
  instances across brick-math + ramsey-uuid) — deliberate mir soundness design
  (`class/overrides.rs:349-376`): callers gate purity enforcement on the receiver's statically-resolved
  `is_pure`, so a silently-dropped re-declaration on override would make that already-shipped
  enforcement unsound. Not a false positive.
- **Narrower docblock param type on an override** (ramsey-uuid `LazyUuidFromString::unserialize`,
  `non-empty-string` vs parent's `string`) — real contravariance/LSP violation, matches real Psalm's
  own rule.
- **`is_string($callable) && function_exists($callable)` / `$callable instanceof Closure`
  RedundantCondition** (brick-math `Assert::callableToClosure`) — real but distinct gap: native
  `callable`-typed values aren't recognized as "could satisfy" `is_string`/`instanceof Closure`
  narrowing (the `TCallable` atom isn't in `narrow_to_string`'s/instanceof narrowing's replace-set).
  Same *flavor* of bug as H1-H4 but touches the narrowing system directly — deliberately not rushed
  in this pass; fits the existing narrowing-gap-audit lineage better than a one-off fix here.
- **webmozart-assert full-baseline classification** (verified against the release binary: unfiltered
  47 issues, baseline applied → clean/exit 0). None are un-suppressed true positives; every finding is
  a legitimate-signal warning that real Psalm flags too, grouped per kind:
    - **`ImpureFunctionCall` ×34**, `src/Assert.php`: `preg_match(...)` regex tests,
      `setlocale(LC_CTYPE, ...)` triads (lines 1433-1696 — deliberate C-locale guard idiom so
      `ctype_alpha` is deterministic), `class_exists`, `is_callable`, `iterator_to_array`,
      `array_map(..., static::valueToString(...))`, `enum_exists(\get_class($value))`, reflection ctor —
      all inside helpers whose public API is documented pure via docblock. mir flags any non-pure
      builtin even when it touches no program state; real Psalm over this design behaves the same.
    - **`ImpureMethodCall` ×4**, `src/Assert.php:1721-1800`: `static::strlen($value)` — delegation to a
      statically-overridable Mixin-provided strlen for purity override purposes; mir sees an unanalyzed
      static call and assumes impurity.
    - **`UndefinedClass` ×5**, `src/PsalmPlugin.php`: all `Psalm\*` classes
      (`PluginEntryPointInterface`, `AfterMethodCallAnalysisInterface`, `PluginRegistrationSocket`,
      `AfterMethodCallAnalysisEvent`, `ExpressionIdentifier`) — dev-only, absent from the runtime
      require; fixture ships without psalm installed. Same optional-dependency lineage as the existing
      `paragonie/random-lib`/`vimeo/psalm` entry above.
    - **`InvalidReturnType` ×2**, `src/Mixin.php:5549,5602`: docblock says
      `@return Closure|callable-string|null` but body returns the caller-passed param typed only
      `mixed callable`. A genuine psalm-style type-lint discrepancy — but it recurs across webmozart's
      100+ auto-generated Mixin methods by design, causes no runtime defect (Severity Error vs native
      return), and is already in the baseline. Real signal against its own docblock contract; not a mir bug.
- **`ForbiddenCode` (`shell_exec`)**, **`UndefinedClass` for optional/require-dev-only deps
  (`paragonie/random-lib`, `vimeo/psalm`, guzzlehttp-guzzle's `LoggerInterface` — `psr/log` is only in
  `composer.json`'s `suggest`, never installed by `--no-dev`)**, **`DeprecatedTrait`/`DeprecatedClass`/
  `DeprecatedMethod` on genuinely `@deprecated` code**, **`RedundantCast` on `uuid_parse()`** (stub
  genuinely returns `string`) — all accurate.
- **`PropertyPossiblyUninitialized`/`PossiblyInvalidArrayAccess` requiring a cross-method static-state
  invariant** (ramsey-uuid) — conservative, no mainstream analyzer proves these either; not chased.
- **`PossiblyInvalidArgument` on unchecked `preg_grep`/`idn_to_ascii`/`dns_get_record` results**
  (egulias-email-validator) — none of the call sites null/false-check before use; matches Psalm's own
  stubs for these builtins.
- **`PossiblyNullMethodCall` calling `getError()` twice and trusting the second call** (egulias-email-
  validator `MultipleValidationWithAnd`) — nothing proves the second call's result matches the first;
  the developer's own `@psalm-suppress PossiblyNullReference` on the line is proof Psalm flags it too.
- **`UnusedParam`/`WrongCaseMethod`** (egulias-email-validator) — a genuinely-unused override param with
  no forcing parent/abstract signature, and a real `validateMXRecord`/`validateMxRecord` casing typo.
- **`UndefinedClass` for Doctrine collections/proxy classes** (myclabs-deep-copy) — `doctrine/
  collections`/`doctrine/persistence` are require-dev-only or not a dependency at all; same
  optional-dependency pattern as the existing `paragonie/random-lib`/`vimeo/psalm` entry above.
- **`PossiblyNullArgument` passing `DatePeriod::getRecurrences(): ?int` into a non-nullable `int`
  param** (myclabs-deep-copy `DatePeriodFilter`) — mir's own stub matches real PHP; real signal.

### Pass 3 additions (phpunit-phpunit + guzzlehttp-guzzle, full-kind sweep)

- **The ~70-80% remainder of phpunit-phpunit's `Mixed*` family, after the `@phpstan-type`/stub-gap fixes
  are accounted for** (superglobals `$_ENV`/`$_SERVER`/`$GLOBALS`; native `mixed`-typed properties/params
  with no docblock refinement; `ReflectionParameter::getDefaultValue()`/`ReflectionFunction::invokeArgs()`/
  dynamic `$obj::$methodName()` calls; `unserialize()`/`json_decode()`/`constant()`; resource-returning
  builtins; `require`; `Throwable::getTrace()`/`debug_backtrace()`) — genuinely untyped by source or by
  design, matches real Psalm.
- **~98% of guzzlehttp-guzzle's `Mixed*` family** (`Client::$config`/`$options`, `CurlFactory`/
  `CurlMultiHandler`'s `$conf`/curl option arrays, `EasyHandle::$options`, `SetCookie::$data`,
  `Middleware.php`'s 7 native-`callable`-typed closures, `PromiseInterface::wait()`/`then()`) — all bare
  `array`/`mixed` by the developer's own docblock or a genuinely-untyped vendored interface; independently
  corroborated by guzzle's own `phpstan-baseline.neon` baselining the identical call sites.
- **`ImpossibleIdenticalComparison: false === curl_multi_init()`** (guzzlehttp-guzzle
  `CurlMultiHandler.php:298`) — mir's `curl_multi_init()` stub never includes `false` for PHP 8+, matching
  the real/JetBrains stub; dead defensive code in guzzle.
- **`ImpossibleIdenticalComparison: SetCookie::getName() !== null`** (guzzlehttp-guzzle
  `CookieJar.php:97`) — `getName()`'s own docblock says non-nullable `string` but its backing data
  genuinely can be null at runtime; an inaccurate docblock in guzzle's own code, not a mir inference bug —
  mir is correctly trusting the stated contract.
- **`RedundantCondition`/`RedundantCast` clusters trusting a docblock/native return type that's simply
  looser than the guarded value in practice** (guzzlehttp-guzzle `SetCookie.php`, `Pool.php`,
  `Middleware.php`, `HostValidator.php`, `CookieJar.php`) — independently corroborated by guzzle's own
  `phpstan-baseline.neon` baselining the identical lines; real signal, not mir-specific.
- **`MissingThrowsDocblock` (78 guzzlehttp-guzzle + 43 phpunit-phpunit sampled instances)** — dominated by
  stub-documented `@throws` on `constant()`/`version_compare()`, genuine rethrows, and direct `throw`
  statements with no docblock; conservative signature-based checking matches real Psalm, same lineage as
  the existing brick-math/webmozart-assert precedent.
- **`MissingClosureReturnType` (31/31, guzzlehttp-guzzle)** and **`UnusedParam` (30/31 guzzlehttp-guzzle,
  9/9 phpunit-phpunit)** — every sampled instance is genuinely untyped/genuinely unused with no forcing
  parent/interface signature; matches the existing egulias-email-validator "genuinely-unused override
  param" precedent.
- **`PossiblyNullMethodCall` (21/21, phpunit-phpunit)** — entirely explained by two already-documented
  patterns: calling the same non-`@psalm-pure` method twice and trusting the second call to match the
  first (egulias-email-validator `MultipleValidationWithAnd` precedent), or an intervening impure call
  correctly invalidating a previously-narrowed property/static fact.
- **`ForbiddenCode` × 2** (guzzlehttp-guzzle `functions.php:27`, `Utils.php:43`, both `var_dump()` inside a
  type-introspection debug helper) — same already-settled `ForbiddenCode` lineage as `shell_exec`,
  different builtin.
- **`TooManyArguments`** (guzzlehttp-guzzle `RetryMiddleware.php:121`, invoking a documented
  `callable(int): int` with 3 args) — genuine signature violation in guzzle's own code (harmless at
  runtime, PHP ignores extra args) — real signal against the documented contract.
- **Two docblock-vs-docblock inconsistencies inside guzzle's own code**
  (`MockHandler.php:56`'s `createWithMiddleware()` vs `__construct()` param strictness;
  `CurlFactory.php:1064`/`CurlShareHandleState.php:116`/`Utils.php:924`'s stub-documented-but-blanket-loose
  builtins `curl_getinfo()`/`curl_share_setopt()`) — inherited stub/upstream-docblock looseness, not a
  mir-introduced gap.
- **`InternalMethod`** (phpunit-phpunit `CodeCoverage.php`, `driverInformation()`) — does not reproduce
  against the current binary/fixture at all, even with `--no-cache` on the full project; stale baseline
  entry, not confirmed as a live finding in either direction — re-baseline and re-check next pass.

---
---

# Real-world compatibility audit — false positives (pass 2)

## STATUS: research complete, not yet fixed. Fix backlog only — landed sectors are removed as they ship;
see `git log`.

Workflow: one sector per commit, `.phpt` fixture each, stash-verified. All repros below are minimal and
abstracted; every "CONFIRMED" item reproduced against the real release binary.

---

## Sector I — Stubs & builtin signatures

**I5 — `array_filter($arr)` (no callback) keeps null/falsy in the value union. CONFIRMED.**
`array_filter(['x' => $nullable, 'y' => 'z'])` still typed `array<…, string|null>` → InvalidReturnType.

**I6 — `Closure::bind` result union (`|false`/`|null`) flags assignment to `Closure`-typed properties.**
Not binary-confirmed standalone; sampled repeatedly. Medium confidence.

**I8 — Param-dependent stub returns not modeled (`hrtime`, `sscanf`). CONFIRMED.**
`hrtime(true)` is `int`, bare `hrtime()` is `array{int,int}`; the single stub signature yields
union → bogus offset diagnostics. Same family as `array_keys()`'s `TKey` fallback (see Sector M below).

**I9 — Deliberate error-type stripping (`preg_replace`, `iconv`) flags the user's own guard. CONFIRMED.**
mir strips `|null`/`|false` from these returns to reduce noise (`call/function.rs`), but then the user's
`=== null`/`=== false` check gets ImpossibleIdenticalComparison — strictly worse than modeling the union.
Strip for flow, but exempt the stripped type from impossibility checks (or keep the union). Additional
repro: `LogicalNot.php:93` (phpunit-phpunit).

---

## Sector J — Subtyping gaps

**J3 — Literal shape rejected by `non-empty-list<T>` when the element needs subtyping. CONFIRMED.**
`array{0: Child<int>}` ⊄ `non-empty-list<Base>` / `non-empty-list<Base<int>>`; `array{0: Gen}` ⊄
`non-empty-list<Gen<int>>`. Exact-class elements pass; plain `list<T>` targets pass. The keyed-shape →
non-empty-list arm uses a stricter element comparison than the list arm. Additional repros:
`ErrorHandler.php:137`, `SourceMapper.php:73,85` (phpunit-phpunit).

**J4 — Bare generic instantiation rejected by a parameterized target. CONFIRMED (shape context).**
`new Gen()` / `new SplObjectStorage()` assigned where `Gen<int>` / `SplObjectStorage<K,V>` declared →
InvalidPropertyAssignment. An unparameterized constructed atom should coerce to the declared binding.

**J5 — `callable(): void` rejects value-returning callables. CONFIRMED.**
```php
/** @param callable(): void $fn */
function f(callable $fn): void {}
f(static fn(): int => 3); // InvalidArgument
```
A void expectation accepts any return (result discarded).

**J6 — `@implements Iterator<int, T>` object rejected by a docblocked `iterable<mixed>` param. CONFIRMED.**
Native bare `iterable` accepts it; `iterable<mixed>` fails against `Traversable<int, Bound>`.

**J7 — Int-range arithmetic and comparison narrowing missing. CONFIRMED.**
`/** @var int<1,max> */ $d = 1; $d *= 2;` → assignment flagged ('int' into 'int<1, max>'). Guard-style
`if ($x < 1 || $x > 100) throw;` also never narrows `int` to `int<1,100>`.

**J9 — `class-string<X>` not accepted by a bare `class-string` template bound. CONFIRMED.**
Missing `(TClassString(Some), TClassString(None))` arm (`subtype.rs` ~308) → InvalidTemplateParam.
Also affects plain argument checks, not just template bounds: `Generator.php:106` (phpunit-phpunit).

**J10 — Unbound free templates `array<TKey, TValue>` rejected where plain `array` expected. CONFIRMED.**
An always-safe upcast; kin of J4 (bare vs parameterized generic).

**J11 — `numeric-string` not coerced to `int|float` params in weak-typing mode. CONFIRMED.**
`ceil((string)($n * 100))` flags in non-`strict_types` files; PHP coerces numeric strings there.
`(string)(float)` casts also lose numericness.

---

## Sector K — Trait composition & member visibility

**K2 — `Closure::bind`/`bindTo` scope argument never feeds closure-body analysis for DYNAMIC scopes.
CONFIRMED residual gap (partially fixed — literal-class-name case already handled).** A scope held in a
variable or parameter — `Closure::bind($fn, $scope, $scope)` where `$scope`'s type isn't a literal
class-string — still analyzes the closure body against the enclosing class instead of the
(unresolvable-at-analysis-time) bound scope, flagging a real member on the intended scope as
`UndefinedMethod` on the enclosing class instead.

---

## Sector L — Inference, narrowing & flow

**L1 — Property fetch on a possibly-null base poisons the result with `|null`. PREMISE LOOKS WRONG —
re-verify before touching.**
```php
private ?Conn $conn = null;
public function f() { return $this->conn->lastId; } // result typed int|string|null
```
Claim: "PHP throws on a null base — the fetch never yields null." That's true for a *method call* on
null (fatal `Error`), but a plain `->` *property fetch* on null is only ever a warning
("Attempt to read property … on null") and evaluates to `null` — it does not throw. `expr/objects.rs`
already emits `PossiblyNullPropertyFetch` (info) at the site AND adds `TNull` to the result
(`analyze_property_access`, ~888-895 and ~947-949), which matches that real runtime behavior: the value
genuinely can be null, so downstream `Nullable*` diagnostics on it are correct, not "bogus". Applying the
suggested fix (drop the `|null`) would make inference *less* accurate, not more. Leave as-is unless a
concrete repro shows the current diagnostics are actually wrong (e.g. double-reporting the same warning
both here and at every downstream use — that would be a real, narrower complaint, but wasn't confirmed).

**L2 — Class-constant union/wildcard docblock types never resolve for comparisons. CONFIRMED.**
```php
/** @param self::MODE_* $mode */
public function run(string $mode): bool { return $mode === self::MODE_DROP; } // "always false"
```
`self::C_A|self::C_B` and `Foo::PREFIX_*` stay opaque atoms; comparisons against member literals flag
ImpossibleIdenticalComparison, and match exhaustiveness over them is wrong. Additional repros: new
manifestation in argument-checking, not just comparison — `BackedUpEnvironmentVariable.php` ×2,
`Phpt/TestCase.php:502,507` (phpunit-phpunit).

**L5 — By-ref out-param write lost in a namespaced closure's `&&` condition. CONFIRMED (composite trigger).**
```php
namespace App;
take(static function (string $data): void {
    if ($data && preg_match('/a(b)/', $data, $m)) { echo $m[1]; } // PossiblyUndefinedVariable $m
});
```
All four parts required: namespace + closure literal passed as an argument + `&&` RHS + unqualified
builtin (`\preg_match` is fine). Likely the global-function fallback is lost when analyzing the closure
body's condition. Sibling of F3.

**L6 — Generic bound to an enum-case type widens back to the whole enum in a closure param. CONFIRMED.**
`Outcome<LoadError::Missing, string>` + `@param Closure(F): Throwable` → a match inside the closure over
the param demands ALL enum cases (native hint wins over the substituted single-case binding).

**L7 — Closure/arrow-fn declared return hint blocks body-inferred literal. CONFIRMED.**
`static fn(): int => 1` passed to `@param Closure():positive-int` flags. Same or-chain priority bug as G5,
manifesting on every callable argument check. Fix together with G5.

**L8 — Variadic param compared element-vs-collected against an inherited docblock. CONFIRMED.**
```php
interface F { /** @param list<non-empty-string> $ids */ public function byIds(string ...$ids): array; }
final class I implements F { public function byIds(string ...$ids): array { return []; } }
// MethodSignatureMismatch: parameter $ids type 'string' is narrower than parent 'list<non-empty-string>'
```
A variadic `@param` documents the collected array; the override comparison mixes the two levels.

**L9 — Un-restated override return still compared against the parent's docblock-refined return. CONFIRMED.**
```php
/** @template T */ abstract class C { /** @return class-string<T> */ abstract public static function k(): string; }
/** @template-extends C<Item> */ final class D extends C { public static function k(): string { return Item::class; } }
// MethodSignatureMismatch: 'string' not subtype of 'class-string<Item>'
```
Also with concrete bindings (`positive-int` via `@template-implements`). A8's bound-level fallback doesn't
cover a bound/concrete-refined parent; a child with no own docblock should inherit, not be compared.

**L10 — `@inheritDoc` on a constructor loses the parent's templated `@param` → class generic never binds. CONFIRMED.**
`new Child(...)` types as bare `Child` instead of `Child<int>`, then fails a `Base<int>`-typed target.
G1-family (inheritDoc `@param` inheritance), constructor + template binding manifestation.

**L12 — Reads inside a diverging catch block don't count as uses. CONFIRMED.**
```php
$w = make();
try { work(); } catch (Throwable $e) { $w->log($e); throw $e; } // UnusedVariable $w
```
Rethrow or `exit` inside the catch triggers it; a falling-through catch is fine.

**L13 — Assertion on a property-chain argument leaks a synthetic variable into unused tracking. CONFIRMED.**
`assertIsStr($result->a->b);` → "UnusedVariable: Variable $result->a is never read" at 1:0. Known E3-era
collision between synthetic 2-hop keys and unused-variable tracking.

**L14 — Stale `/** @var T $name */` for a never-created variable reported as UnusedVariable at 1:0. CONFIRMED.**
Should be ignored (or an InvalidDocblock-class notice at the docblock's own line).

**L15 — By-ref out-binding makes a foreach value "unused". CONFIRMED.**
`$stmt->bindParam(':d', $data); foreach ($rows as $id => $data) { $stmt->execute(); }` →
UnusedForeachValue — the loop variable is read through the bound reference.

**L16 — Unterminated generic docblock kept as a literal type string. CONFIRMED.**
`@var Wrap<Item` (missing `>`) produces a type that can never match ('Wrap<Item'); should fall back to the
bare class or flag the docblock itself.

**L17 — PHP 8.4 get-hook virtual property flagged PropertyPossiblyUninitialized. CONFIRMED.**
```php
public int $doubled { get => $this->value * 2; } // "may be left uninitialized by the constructor"
```
Hooked (especially virtual, get-only) properties have no backing store to initialize. F7 sibling.

**L18 — Existence-guard narrowing family. CONFIRMED (multiple forms; `property_exists`/`isset` form
already fixed).**
```php
if (class_exists(NewApi::class)) { NewApi::run(1, 2); }        // dead-branch BC calls still hard-error
```
(b) `class_exists`/`interface_exists`/`method_exists` with LITERAL class args aren't const-evaluated, so
version-gated BC branches flag TooFew/TooManyArguments/InvalidArgument; (c) `method_exists` facts are
consulted only on direct-call paths — `[$this, 'm']` array-callables and closures still flag. The
`defined('X::Y')` half of a related static-const-guard form has no tractable target yet — there is no
`UndefinedClassConstant`-style diagnostic in `mir_issues::IssueKind` to suppress in the first place, so
this narrowing would have nothing to feed; lowest priority, revisit only if such a diagnostic is ever
added. Explained 86% of one run's TooFewArguments plus the largest UndefinedProperty cluster of another.

(c), the array-callable/closure gap, is the more tractable of the two remaining: `validate_callable_type`
(`call/args/types.rs:1445-1532`, called unconditionally from `check_one`) emits `UndefinedMethod` off a bare
`find_method_in_chain` lookup with no `FlowState`/`ctx` and no receiver AST expr in scope — no path to
`method_exists_guards` or method.rs's other suppressions (interface/abstract/trait/`__call`). Needs `ctx`
and the receiver `Expr` threaded through `check_one`/`check_args`/`ArgBinding` (3-4 signatures) to reuse the
same suppression logic. (b) is the harder of the two: neither `check_counts` nor its callers
(`call/args/counts.rs`, `call/args.rs:419`) ever consult `class_exists_guards`/`is_class_guarded` — arity/
type checking runs unconditionally once `resolve_method_from_db` succeeds, guard or not, and there's no
existing "skip checks for this receiver" hook to reuse; would need a real design decision spanning both
`method.rs` and `static_call.rs`'s resolved-method success branches.

**L20 — Fully-qualified global names (`\Foo`) in namespaced files break member lookup on intersection
receivers and by-ref premarking. CONFIRMED (1 of 4 forms remaining; 3 already fixed via
`db::resolve_receiver_fqcn`).**
`call/static_call.rs::extract_object_fqcn` has no `TIntersection` arm at all (falls to `_ => return None`,
i.e. static-call resolution on an intersection receiver is skipped outright — a missing-feature/
false-negative gap, not the same double-resolution pattern, since this function never calls
`resolve_name` on its atoms to begin with). Fixing it means threading `db`/`file` into
`extract_object_fqcn` (currently `fn(ty: &Type) -> Option<String>`, no such params) so a `TIntersection`
part's `named_object_fqcn()` result can go through `resolve_receiver_fqcn` — a signature change across
its call site(s), not a drop-in swap. Deferred pending a concrete repro (`\Glob`/intersection receiver on
a STATIC call, e.g. `\Glob::method()`-typed intersection member, in a namespaced file).

**L21 — `static` inside generic args concretized on `$this` calls. CONFIRMED.**
`@return G<static>` invoked on `$this` yields `G<Concrete>`, then fails against a declared `G<static()>`
(`call/args.rs::substitute_static_atom` rewrites nested `TStaticObject` unconditionally). ~28 items on
one run across InvalidArgument/InvalidReturnType.

**L22 — `new static` typed as the concrete class, not `static`. CONFIRMED.**
Breaks the singleton `static::$instance ??= new static()` idiom (expects 'static()', got the class).
`expr/objects.rs` builds `TNamedObject` instead of `TStaticObject` (~341/706).

**L23 — Conditional return types with literal-string predicates unresolved. CONFIRMED.**
`@return (T is 'array' ? A : B)` with `T` defaulted/bound to a literal never picks a branch — the whole
union flows on and each member's missing method flags (`call/mod.rs::resolve_conditional_return`).

**L24 — `@param-closure-this` tag unsupported. CONFIRMED.**
Closure bodies registered via a `macro()`-style API analyze `$this` as the provider class.

**L25 — Calls on an unresolvable receiver type cascade into UndefinedMethod. Policy.**
A docblock-typed receiver whose class isn't loadable (optional dep) emits no UndefinedClass but flags
EVERY method call. Suppress member checks on unresolvable receivers (report the class once, if at all).

**L26 — Union receiver: per-atom arg/arity errors with no sibling-accepts suppression. CONFIRMED.**
When one union atom's signature accepts the call, a sibling atom's mismatch still hard-errors
(`call/method.rs::resolve_method_return` per-atom loop). Non-`__call` analogue of fixed A6/C9.

**L29 — `array_merge`/`array_unique` lose element types; bare `array` downcast policy. CONFIRMED (partial).**
Merged/uniqued arrays widen to bare `array`, then flag against typed array properties. Additional
repros: `PassedTests.php`, `GroupFilterIterator.php`, `Runtime/PHP.php` (phpunit-phpunit).

**L31 — By-ref writes through callable VALUES lost. CONFIRMED.**
`$fn = 'preg_match'; $fn($re, $s, $m);` (and Closure-valued callables) never premark `$m`. Distinct from
L5/F3 (~10 instances on one run).

**L33 — Array-shape gaps (B11 family, new forms). CONFIRMED (2 of 3).**
`$arr + static::CONST_ARRAY` loses the constant's keys; tuple-discriminant narrowing
(`if ($t[0] === 'x') use($t[1])`) missing; cross-method property shapes treated closed (unpinned).
Also: post-loop state drops continue-path values (loop counters stay literal `0`). Additional repros:
`Cli/Builder.php` ×8, `SourceMapper.php:247`, `Issue.php:72`, likely `Generator.php:528` (phpunit-phpunit).

**L34 — Parser: `include \dirname(__DIR__)` parsed as a call to `include\dirname`. CONFIRMED.**
Upstream php-ast crate; niche but produces bogus UndefinedFunction.

**Open question — lazy vendor loading may drop interface method tables. UNPINNED.**
One vendor-interface UndefinedMethod cluster (13 instances) reproduces only in the full-project run —
faithful standalone and monorepo repros are clean. Suspect the lazy classmap path loses methods; needs
dedicated pinning before any fix.

---

## Sector M — Harness pass 3 (full-coverage sweep, phpunit-phpunit + guzzlehttp-guzzle)

7 parallel audits swept every remaining diagnostic kind in both baselines to completion (882 + 787 =
1669 diagnostics). Most CONFIRMED findings from this pass have since landed (see `git log`); the
still-open remainder:

**M7 — `array_keys()`'s `TKey of int|string` template falls back to its full bound when the source
array's key type is unknown. Landed (sibling of I8).** Two independent repro paths, both fixed:
(a) `get_defined_constants()`'s stub declared a bare `array`, so `array_keys(...)`'s `TKey of
int|string` fell back to its full bound — the stub now says `@return array<string, mixed>`
(`stubs/Core/Core.php`); (b) guzzle `SetCookie::$defaults` is a bare `@var array` **private static**
property whose literal initializer is string-keyed — the collector now types literal property
initializers (`collector/literal_types.rs`) and refines a `private static` property's declared type
into the literal's shape when the initializer is a typed literal and no in-file write mutates the
property (a `private static` is file-scoped, so that scan is complete); written or non-private-static
properties keep their declared type, and untyped initializers keep the historical `mixed`. Regression
fixtures in `tests/fixtures/by-kind/possibly_invalid_argument/` (two positive, two gate negatives).
guzzlehttp-guzzle `--show-info` diff vs HEAD: exactly the two M7 lines (`src/Utils.php:894`,
`src/Cookie/SetCookie.php:70`) removed, zero additions; default-level output byte-identical
(MIR0105 is info-severity — invisible at the default CLI level, so the fixtures are the durable
guard).
Same family as I8 (param/shape-dependent stub imprecision).

**M19 — Concatenation with a provably-non-empty operand (literal prefix, `DIRECTORY_SEPARATOR`) isn't
inferred non-empty overall. CONFIRMED — generalization of the existing carryover B8 (scoped to
interpolated strings) to the explicit `.` operator.** phpunit-phpunit: `Filesystem.php:46`,
`TestSuiteFilterProcessor.php:82`.

**M29 — `ini_get_all()`'s stub declares a bare `array|false` return; its `#[ArrayShape]` attribute (which
mir doesn't parse at all) is decorative only. CONFIRMED.** Outer key infers as `int|string` (default)
instead of `string`, and the value infers `mixed` instead of the documented per-setting shape. Repro:
`foreach (ini_get_all(...) as $key => $value) { needsString($key); }` fires `PossiblyInvalidArgument`
(`int|string` provided). Fix: replace the attribute-only doc with a real
`@return array<string, array{global_value: string, local_value: string, access: int}>|false` docblock,
matching the working precedent at `stubs/date/date.php:1415` and `stubs/imagick/imagick.php:5232`.
phpunit-phpunit: `Util/GlobalState.php:252`. No existing fixture references `ini_get_all` — unconstrained.

**Still unconfirmed (not yet investigated):**
- Two `Resolver::resolve()` array-shape param docblocks losing optional keys / collapsing a
  literal-string-union field to `mixed` against a same-file `@phpstan-type` alias — possibly a docblock
  array-shape/alias-expansion gap. `ErrorHandler.php:630,706` (line numbers may have shifted; not
  re-verified against current HEAD).
- `PossiblyUndefinedVariable` (2 instances) and `TestSuite.php:464,468` /`Merger.php:424,425`/
  `Loader.php:314` (structurally identical to the L33 loop-carried-write family but not independently
  minimized) — not chased given time budget.

**Additional confirmations of existing carryover/sector items, logged here for traceability (not
independently new):** **B3** — all 10 `InvalidTemplateParam` instances plus 2 `JunitXmlLogger.php`
PropertyTypeCoercion + 4 Migration-file `PossiblyInvalidArgument` (`DOMElement|false`). **B11/L33** —
guzzlehttp-guzzle `Utils.php:894` (array_keys, see M7), `Client.php:109`/`Utils.php:222` (array-shape
widening via `foreach`), `StreamHandler.php:597,601` (`NonExistentArrayOffset` ×2, nested-shape key
loss), `RedirectMiddleware.php:220` (cross-key value contamination), `CookieJar.php:269`
(loop-accumulator lost with a preceding `continue`). **E4/E5** — `CurlFactory.php:1021` (narrowing lost
after a preceding `isset`+`try`/`catch` block).

---

## Carryover — still-open pass-1 sectors (compressed; details in pass-1 history)

- **Sector 0** — plugin bridge can't use an isolated tooling vendor autoload; `autoload` RPC param exists
  in the PHP host but is never sent from Rust. Config knob + wire-through.
- **B3** — stale DOM stub: `createElement()` family still `T|false` for PHP ≥ 8.1.
- **B4** — `self`/class-level `@template T` unresolved inside container types (`list<self>` in `@var`/
  `@return` — re-confirmed this pass: `@return list<self>` on a static method flags against `list<self()>`).
- **B5** — `Enum::Case` docblock types resolved without `use` imports.
- **B6** — fixed for the common case (`subtype.rs`'s `(TIntersection, b)` arm). Residual scope, if any,
  not yet found.
- **B8** — interpolated string with literal parts not `non-empty-string`.
- **B9** — bare inline `@var Type` (no variable name) cast unapplied.
- **B10** — `preg_match` named capture groups not modeled (mir currently types `$matches` as a plain
  `list<string>`/`list<array{0:string,1:int}>` regardless of named groups — see L3's fix note in git log
  for a case where this leniency was deliberately preserved rather than papered over).
- **B11** — array-shape tracking degrades across loop/conditional writes.
- **B12** — nested array-shape class-string covariance not recursive.
- **C1-remnant / G12** — unbindable `class-string<T>` falls back to literal `T` → `UndefinedClass: T`.
- **C11** — literal string into `class-string<T>` param over a native `string` hint (IoC-key idiom).
- **D6** — `TypeDoesNotContainType` only covers match/switch subjects.
- **D8** — cross-tool suppression name aliasing (`PropertyNotSetInConstructor` ↔ `PropertyPossiblyUninitialized`).
- **E4** — narrowing lost through a method-call receiver (`if ($x->get() instanceof Y) { $x->get()->m(); }`
  — re-confirmed this pass as a top UndefinedMethod source).
- **E5** — property narrowing capped at 1 hop for every condition check (systemic; own audit pass).
- **F1** — docblock generic ARGUMENT class names not FQCN-resolved (re-confirmed: `@param list<Item>` on a
  non-constructor method stays `list<Item>` vs `list<Ns\Item>`; constructor path resolves).
- **F2** — cross-file `@psalm-import-type` never reaches `@param` resolution (re-confirmed; also cascades
  into InvalidOperand on the unresolved alias).
- **F3** — by-ref writes in `&&`/`||` RHS lost for pre-existing variables (see also L5).
- **F4** — `count()` narrowing misses keyed-shape atoms.
- **F5** — `isset($arr['k']->prop)` doesn't narrow the key's presence.
- **F7** — `PropertyPossiblyUninitialized` understands only the constructor (design pass; see also L17).
- **G1** — `@inheritDoc` inherits `@return` only, never `@param` (see also L10).
- **G2** — recursive `list<self>` loses type after the first hop.
- **G3** — `@template-implements` binding not substituted into an override's native `mixed` param.
- **G4** — `array_map`-family callbacks: untyped params not seeded from the iterable's element type.
- **G5** — any declared native return type blocks a narrower body-inferred type (`resolve_fn` or-chain;
  fix together with L7).
- **G6** — `ArrayAccess<TKey,TValue>` generic binding unused for offset reads.
- **G8** — untyped closure param not bound from the receiver's generic (`Closure(TValue):T` idiom).
- **G10** — PHP 8.4 property hooks on interfaces don't register properties.
- **G11 — PARTIAL.** `instanceof`/`is_*`/`ctype_*` narrowing on a literal-keyed array-offset access now
  works (true/false branches, nested paths, property/static-property receivers — see git log). Still
  missing: the `strlen`/`count` comparison families and enum-class narrowing have no array-offset arm; a
  generic `array<K,V>`/`list<T>` base (no per-key storage) is also still unnarrowed by any of these.
- **G13** — MixedArrayAccess/MixedAssignment/MixedFunctionCall still need their full per-kind pass.

---

## Fix plan (suggested order)

0. **Sector M remainder**: M19 (fold into B8 lineage), M29 (stub docblock fix), plus the two
   "still unconfirmed" items.
1. **L18** (existence guards, (b)/(c) remaining) + **L20** (FQ-global member lookup, 1 of 4 remaining) —
   each explains a top cluster in its kind.
2. **L1–L2** (null-poisoned fetch — re-verify premise first for L1; const-union comparisons for L2).
3. **K2**, **L5–L17**, **L21–L34** — small, independently testable; batch by file touched
   (L21+L22 share the static-substitution area; L5+L31+F3 share by-ref premarking).
4. **I5–I9** (stub signatures) + **B3** together.
5. **J3–J7, J9–J11** (subtyping) — J3/J4/J10 likely share a fix point.
6. Pin the lazy-vendor method-table question before touching vendor loading.
7. Carryovers per pass-1 order: G11 remainder (strlen/count/enum-class array-offset arms), E5 (dedicated
   audit), F1/F2 (docblock resolution family), rest opportunistically.

---

# Cache/index audit (2026-08-04, post-71d20034; pass 2 2026-08-05)

## STATUS: pass 2 landed X5/X7-partial and closed X4/X11 with measurements; new correctness fix
(subtype-edge epoch in the query-memo keys) landed alongside. Pass 1 sweep cross-checked against
php-lsp's actual usage (php-lsp HEAD consumes 71d20034's raw-needle APIs at
`document_store.rs:1840-1843`). Landed/closed items summarized below, then the remaining backlog.

## Landed / closed (pass 2, 2026-08-05) — see `git log` for X1, X2, X3, X9

- **X5 — LANDED.** `ref_query_cache`/`subtype_query_cache` are now `RevisionedMap`s: keys embed a
  `(text_revision, subtype_edges_epoch)` generation and the first insert at a newer generation drops
  the dead one wholesale (`session/mod.rs` `RevisionedMap`). Regression-tested (counter assertions in
  `write_path_hoist.rs`).
- **NEW, landed with X5 — within-revision staleness fix.** Both memo caches keyed only on
  `text_revision`, but subtype edges + anonymous `impl:`/`implshort:` postings mutate off-salsa
  (`set_file_class_edges` / `set_file_reference_locations`): a member-references result cached before
  a subtype BFS committed a gate-invisible file's edge served a smaller hierarchy fan-out forever
  within that revision. Now `MirDbStorage::subtype_edges_epoch` bumps on real edge changes only
  (unchanged recommits compare equal and skip — so background sweeps don't churn it) and both cache
  keys carry it. Pinned by
  `write_path_hoist.rs::references_cache_invalidates_when_subtype_query_grows_hierarchy`
  (verified to fail with the epoch frozen).
- **X4 — CLOSED, keep `lru = 256`.** Measured on the Laravel fixture (11.6k registered files):
  `lru = 65536` moved neither wall (cold Str query 0.748s vs 0.755s; cold CLI batch 1.96s vs 2.24s,
  both within noise) nor peak RSS (~471 MB both paths). The Phase-2 re-parse hides inside the rayon
  pass; reference queries only ever parse the gate-admitted subset. Verdict recorded on the query's
  doc comment.
- **X7 — PARTIAL.** The drift risk is gone: the tracked `workspace_symbol_index` fallback now drives
  the same `tier_insert`/`tier_insert_class_like` helpers as the imperative rebuild/seed/merge paths
  (one precedence encoding). Full deletion of the tracked fn + `workspace_index()` fallback still
  blocked: raw-`MirDbStorage` unit tests and the pre-first-rebuild window in a live session rely on
  it. `workspace_classes`/`workspace_functions` kept: real batch consumers, and retiring them changes
  enumeration order + duplicate-FQCN visibility (see backlog).
- **X11 — CLOSED, keep serial.** Re-attempted 2026-08-05 with X3's deferred-bump scope in place
  (rayon `try_for_each` variant): `concurrent_reference_cancel` still deadlocks (>590s for a ~5s
  test, CPU ~0). The revision-bump storm was NOT the trigger; pool-saturation on `with_db_mut`'s
  write lock stands. Documented at the Phase 1 loop. Any future attempt needs a different execution
  model (own pool / async), not a rayon swap.
- **R3 (reported by sweep) — CHECKED FALSE.** Mention-index entries ARE evicted: text replace
  (`mirdb.rs` `upsert_source_file_with_durability`) and `remove_source_file` both call
  `clear_file_class_mentions`; entries pin only current-text `Arc`s (refcounts, not copies).

## Remaining backlog

### Perf (user-visible)

- **X10 — delta-merge upgrade for novel-needle recording passes.** When a file has a
  current-text mention entry at an older epoch, the gate rescans it against the WHOLE universe
  (66ms/12.7k files, 44 MB transient churn). A narrow scan of just the delta needles merged into the
  existing entry (append-only universe makes the merge sound: entry.names ∪ delta-hits at the new
  epoch equals a full rescan) would cut that to ~3ms. Needs a per-entry-epoch delta scanner and
  care with concurrent `set_file` races. Only worth it if profiling shows novel-needle queries
  hot in real hosts — the cost is once per distinct queried symbol name per session.
- **X12 — short-name lookups that linear-scan.** `class_like_by_short_name` exists, but:
  `body_analysis/mod.rs:254` linear-scans `workspace_functions` comparing short names (metric
  `record_fn_short_name_scan` counts it); `call/args/types.rs:747`/`:776` linear-scan
  `workspace_classes`; `SubtypeIndex::subtypes_of_lenient` (`subtype_index.rs:277-283`) re-derives
  short-name roots by scanning every `children` key. A `functions_by_short_name` bucket (same
  lockstep maintenance as the class one) + a short-name map in `SubtypeIndex` kills all three.
  The two body-analysis scans also compare case-SENSITIVELY (`==`) — PHP function/class names are
  case-insensitive, so these are missed-match bugs as well as scans.

### Correctness (verified this pass, fix with tests)

- **X13 — `extends_or_implements` compares class names byte-exactly** (`db/queries.rs:437`, `:442`,
  `:455`): `child == eff` / `interfaces().any(|i| i.as_ref() == eff)` /
  `ancestors.any(|p| p.as_ref() == eff)` on display-form names. Differently-cased spellings of the
  same class produce false negatives, and the `SubtypeCache` layered on top caches per-spelling
  (hashed raw strings), so the same pair can hold different answers. Normalize with
  `eq_ignore_ascii_case` (or lowercase at entry) and key the cache on the normalized pair.
- **X14 — `InferredFileTypes` keys are case-sensitive-as-written** (`db/queries.rs` map docs): the
  file is found via the LOWERCASED index (`inferred_types.rs`) but the per-file map is keyed by the
  caller's raw FQCN/FQN spelling — a differently-cased call site misses the inferred type. Lowercase
  the class/function halves at insert and lookup (property names stay case-sensitive).

### Hygiene / dedup (reported by the 2026-08-05 sweep; verify before fixing)

- **X6 — mention universe is append-only** (`class_mention_index.rs:148-151`) — fine while it held only
  declared class short-names; hosts now inject arbitrary literal + raw needles, and each novel needle
  rebuilds the automaton and epoch-invalidates older per-file entries. Watch `scanner_bytes`/rebuild
  churn on long sessions before adding machinery.
- **X8 — `last_ingested_symbols`** (`session/mod.rs:65`) duplicates `file_decl_snapshots` in spirit,
  but a direct merge is BLOCKED: the diff's consumers (`stale_defined_symbols` →
  `symbol_referencers_of` probes with `cls:`/`fn:`/`gcnst:` prefixes, `incremental.rs:537-552`) need
  display-form untagged names to match `RefIndex`'s case-sensitive posting keys, while
  `file_decl_snapshots` stores lowercased kind-tagged `Name`s. Unblocks if/when the posting-key
  namespace is case-normalized (X15).
- **X15 — reference-key namespace mixes case conventions.** `RefIndex` keys are display-form
  case-sensitive (`Name::codebase_key`) except `impl:`/`implshort:` which are lowercased; method
  halves of `meth:` keys are lowercased but class halves aren't. Queries compensate ad hoc
  (lowercase-dedup of hierarchy lists). One convention (lowercase class/function halves everywhere)
  would also unblock X8 and delete compensation code.
- **X16 — `class_ancestors` (`ancestors.rs:38`) near-duplicates `class_ancestors_by_fqcn`**
  (`find_queries.rs:868`): both tracked, both `Fqcn`-keyed, both walk `find_class_like`; differ only
  in ordering/self-inclusion/enum-trait handling. Two callers left (`body_analysis/classes.rs`,
  `class/mod.rs`) — migrate them, delete the query and its memo table.
- **X17 — `RefIndex` mints a private `FileNo` path-id space** (`ref_index.rs:33-35`) while
  `SourceFile` + `source_files` already intern paths; `SubtypeIndex`/`ClassMentionIndex` key by raw
  `Arc<str>`. Three representations of file identity; RAM is refcounts (small) but every cross-index
  join goes through strings.
- **X18 — the ref-posting serial-commit block is copy-pasted** (`session/queries.rs` Phase-2 commit,
  `session/incremental.rs` sweep commit; a thinner variant in `ingest.rs::commit_file_refs`). Factor
  one commit helper — behavior-preserving, kills the drift risk between the query and sweep paths.
- **X19 — global constants have no indexed `Location`** (`StubSlice.constants` is `(name, Type)`):
  goto-definition falls back to a per-call raw-text line scan (`global_constant_decl_range`), and
  `document_symbols` emits `location: None` for constants. Adding a `Location` to the tuple (or a
  parallel `Vec`) removes a whole text-scanning path; touches the slice format → disk-cache version
  bump (`$TMPDIR/mir-fixture-stub-cache` staleness applies).

## Checked, keep as-is

- `ref_committed`/`defs_committed`/`prepared_files`/mention `by_file` each pin the same per-file
  `Arc<str>` — pointers, not copies; merging them is a complexity cleanup, not a RAM win.
- `ParseCache` / `StubSliceCache` / `collect_file_definitions` memo — three tiers sharing one
  `Arc<StubSlice>`, no payload duplication.
- The two query memos' value caps (~5 MB + ~4 MB) are a good RAM-for-latency trade; keep.
- `SubtypeEntry.location` duplicates the def-struct locations, and `SubtypeIndex.decls`/`by_file`
  share `Arc<SubtypeEntry>`s — the location copy is what lets subtype queries answer without
  loading slices; keep.
- Mention-index footprint at Laravel scale: ~2.7 MB entries + 1.6 MB scanner for 11.6k files
  (bench print) — proportional and bounded; keep.
