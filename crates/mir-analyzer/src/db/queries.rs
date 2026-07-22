use std::sync::Arc;

use mir_codebase::definitions::TemplateParam;
use mir_issues::Issue;
use mir_types::{Location, Name, Type};
use rustc_hash::{FxHashMap, FxHashSet};

use super::*;
use crate::db::SourceFile;

/// Snapshot of a class's discriminator + abstractness.
#[derive(Debug, Clone, Copy)]
pub struct ClassKind {
    pub is_interface: bool,
    pub is_trait: bool,
    pub is_enum: bool,
    pub is_abstract: bool,
}

pub fn class_kind(db: &dyn MirDatabase, fqcn: &str) -> Option<ClassKind> {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    let class = crate::db::find_class_like(db, here)?;
    Some(ClassKind {
        is_interface: class.is_interface(),
        is_trait: class.is_trait(),
        is_enum: class.is_enum(),
        is_abstract: class.is_abstract(),
    })
}

pub fn class_exists(db: &dyn MirDatabase, fqcn: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    crate::db::find_class_like(db, here).is_some()
}

/// True when `fqcn` is a `final` class (or an enum, which is implicitly
/// final) — i.e. provably has no subclasses. Unknown classes are not final.
pub fn is_final(db: &dyn MirDatabase, fqcn: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    crate::db::find_class_like(db, here).is_some_and(|c| c.is_final())
}

pub fn function_exists(db: &dyn MirDatabase, fqn: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqn);
    crate::db::find_function(db, here).is_some()
}

pub fn constant_exists(db: &dyn MirDatabase, fqn: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqn);
    crate::db::find_global_constant(db, here).is_some()
}

pub fn resolve_name(db: &dyn MirDatabase, file: &str, name: &str) -> String {
    if name.starts_with('\\') {
        return name.trim_start_matches('\\').to_string();
    }

    let lower = name.to_ascii_lowercase();
    if matches!(lower.as_str(), "self" | "static" | "parent") {
        return name.to_string();
    }

    if name.contains('\\') {
        if let Some((first, rest)) = name.split_once('\\') {
            let imports = db.file_class_imports(file);
            if let Some(base) = imports.get(&Name::new(first)) {
                return format!("{}\\{rest}", base.as_str());
            }
            // Case-insensitive fallback: PHP resolves the leading segment of a
            // qualified name against `use` imports case-insensitively, same as
            // unqualified names below.
            if let Some((_, base)) = imports
                .iter()
                .find(|(alias, _)| alias.as_str().eq_ignore_ascii_case(first))
            {
                return format!("{}\\{rest}", base.as_str());
            }
        }
        // If the name is already a known FQCN (e.g. stored in TNamedObject by a prior
        // resolution step), return it unchanged to avoid double-prepending the namespace.
        if class_exists(db, name) {
            return name.to_string();
        }
        // Qualified name not yet in the DB (PSR-4 lazy-load will fire after this call):
        // PHP always prepends the current namespace unconditionally.
        if let Some(ns) = db.file_namespace(file) {
            return format!("{}\\{}", ns, name);
        }
        return name.to_string();
    }

    let imports = db.file_class_imports(file);
    if let Some(fqcn) = imports.get(&Name::new(name)) {
        return fqcn.as_str().to_string();
    }
    // Case-insensitive fallback: PHP class names are case-insensitive for
    // resolution. Iterate as a last resort; the exact-case hit above
    // catches the common path.
    if let Some((_, fqcn)) = imports
        .iter()
        .find(|(alias, _)| alias.as_str().eq_ignore_ascii_case(name))
    {
        return fqcn.as_str().to_string();
    }
    if let Some(ns) = db.file_namespace(file) {
        return format!("{}\\{}", ns, name);
    }
    name.to_string()
}

/// The `@template` params `fqcn` itself declares in its own docblock — empty
/// (or `None`) if it declares none, even when an ancestor further up the
/// chain does. Only the handful of callers that specifically care about
/// *this type's own declaration* (e.g. comparing a subclass's declared arity
/// against an ancestor's) should reach for this; almost everyone else wants
/// [`class_template_params`], which resolves the inherited case too.
pub fn declared_template_params(db: &dyn MirDatabase, fqcn: &str) -> Option<Arc<[TemplateParam]>> {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    let class = crate::db::find_class_like(db, here)?;
    Some(Arc::from(class.template_params().to_vec()))
}

/// The `@template` params that parameterize `fqcn` — its own, or if it
/// declares none, the nearest ancestor's found by walking up its native
/// `extends` chain. A plain subclass that doesn't redeclare `@template`
/// (`class IntBox extends Box {}`) is still implicitly parameterized the
/// same way its generic ancestor is — PHP/Psalm don't require re-declaring
/// an un-narrowed inherited template. This is what almost every caller wants
/// ("does this receiver have template slots to bind" — constructor-arg
/// inference on `new`, building bindings for a method call); reach for
/// [`declared_template_params`] instead only when own-declaration-only is
/// specifically the question being asked.
pub fn class_template_params(db: &dyn MirDatabase, fqcn: &str) -> Option<Arc<[TemplateParam]>> {
    let mut visited: FxHashSet<Arc<str>> = FxHashSet::default();
    // A worklist, not a linear `current = parent` chain: a bare interface
    // (`interface DogContainer extends AnimalContainer {}`) may extend
    // several bases at once, and PHP/Psalm don't require it to redeclare an
    // un-narrowed inherited `@template` — same reasoning `declared_template_params`'s
    // doc comment already gives for the class case, which this used to be the
    // only branch for (see `inherited_template_bindings` for the analogous
    // worklist over interfaces' multi-base `extends`).
    let mut worklist: Vec<Arc<str>> = vec![Arc::from(fqcn)];
    while let Some(current) = worklist.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(tps) = declared_template_params(db, current.as_ref()) {
            if !tps.is_empty() {
                return Some(tps);
            }
        }
        let here = crate::db::Fqcn::from_str(db, current.as_ref());
        match crate::db::find_class_like(db, here) {
            Some(crate::db::ClassLike::Class(cls)) => {
                if let Some(parent) = cls.parent.clone() {
                    worklist.push(parent);
                }
            }
            Some(crate::db::ClassLike::Interface(iface)) => {
                worklist.extend(iface.extends.iter().cloned());
            }
            _ => {}
        }
    }
    None
}

pub fn inherited_template_bindings(
    db: &dyn MirDatabase,
    fqcn: &str,
    own_bindings: &FxHashMap<Name, Type>,
) -> FxHashMap<Name, Type> {
    let mut bindings: FxHashMap<Name, Type> = FxHashMap::default();
    let mut substitution: FxHashMap<Name, Type> = own_bindings.clone();
    let mut visited: FxHashSet<Arc<str>> = FxHashSet::default();
    // A worklist, not a linear `current = parent` chain: an interface's
    // native `extends A, B` clause may name several bases at once, and each
    // of THOSE may further extend other generic interfaces — walking only
    // the class/parent spine (as this used to) silently drops any template
    // parameterization declared past the first interface hop.
    let mut worklist: Vec<Arc<str>> = vec![Arc::from(fqcn)];

    let apply_type_args = |iface: &Arc<str>,
                           args: &[Type],
                           bindings: &mut FxHashMap<Name, Type>,
                           substitution: &mut FxHashMap<Name, Type>| {
        let Some(iface_tps) = class_template_params(db, iface.as_ref()) else {
            return;
        };
        for (tp, ty) in iface_tps.iter().zip(args.iter()) {
            let resolved_ty = ty.substitute_templates(substitution);
            substitution
                .entry(tp.name)
                .or_insert_with(|| resolved_ty.clone());
            bindings.entry(tp.name).or_insert(resolved_ty);
        }
    };

    while let Some(current) = worklist.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        let Some(class) =
            crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, current.as_ref()))
        else {
            continue;
        };

        for (iface, args) in class.implements_type_args() {
            apply_type_args(iface, args, &mut bindings, &mut substitution);
        }
        for (iface, args) in class.interface_extends_type_args() {
            apply_type_args(iface, args, &mut bindings, &mut substitution);
        }
        if let Some(parent) = class.parent() {
            let extends_type_args = class.extends_type_args();
            if !extends_type_args.is_empty() {
                apply_type_args(parent, extends_type_args, &mut bindings, &mut substitution);
            }
        }

        // A used trait's own `@template` params are bound from an explicit
        // `@use TraitName<T>` type-argument list when the using class/trait
        // supplies one, substituting any of ITS OWN templates already bound
        // above/by the caller (`@use Collection<self>`-style forwarding);
        // otherwise (or for an arg position past the supplied list) they
        // fall back to `mixed` — the same fallback un-bound templates get
        // elsewhere — rather than staying absent: an absent entry would let
        // the RECEIVER's own same-named template letter silently leak into
        // the trait's property/method types wherever this function's result
        // is merged with a caller's own bindings via `entry().or_insert()`.
        for trait_fqcn in class.class_traits() {
            if let Some(trait_tps) = declared_template_params(db, trait_fqcn.as_ref()) {
                let explicit_args = class
                    .trait_use_type_args()
                    .iter()
                    .find(|(t, _)| t == trait_fqcn)
                    .map(|(_, args)| args.as_slice());
                for (i, tp) in trait_tps.iter().enumerate() {
                    let resolved = explicit_args
                        .and_then(|args| args.get(i))
                        .map(|ty| ty.substitute_templates(&substitution))
                        .unwrap_or_else(Type::mixed);
                    substitution
                        .entry(tp.name)
                        .or_insert_with(|| resolved.clone());
                    bindings.entry(tp.name).or_insert(resolved);
                }
            }
        }

        // Keep walking every ancestor edge — typed or not — so a base
        // interface/class/trait that itself parameterizes (or uses) a
        // further generic ancestor still gets picked up even when the edge
        // leading to it carried no type args of its own.
        worklist.extend(class.interfaces().iter().cloned());
        worklist.extend(class.extends().iter().cloned());
        worklist.extend(class.class_traits().iter().cloned());
        if let Some(parent) = class.parent() {
            worklist.push(parent.clone());
        }
    }
    bindings
}

pub fn has_unknown_ancestor(db: &dyn MirDatabase, fqcn: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    if crate::db::find_class_like(db, here).is_none() {
        return false;
    }
    crate::db::class_ancestors_by_fqcn(db, here)
        .iter()
        .skip(1) // self
        .any(|ancestor| !class_exists(db, ancestor))
}

pub fn member_location(db: &dyn MirDatabase, fqcn: &str, member_name: &str) -> Option<Location> {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    // `find_method_respecting_precedence` (not the plain `find_method_in_chain`
    // walk) so go-to-def resolves a trait-aliased method name (`use T { foo as
    // bar; }`) and picks the `insteadof`-winning copy on a trait conflict —
    // both invisible to a plain own-methods lookup on the using class.
    if let Some((_, storage)) = crate::db::find_method_respecting_precedence(db, here, member_name)
    {
        if let Some(loc) = storage.location.clone() {
            return Some(loc);
        }
    }
    if let Some((_, storage)) = crate::db::find_property_in_chain(db, here, member_name) {
        if let Some(loc) = storage.location {
            return Some(loc);
        }
    }
    if let Some((_, storage)) = crate::db::find_class_constant_in_chain(db, here, member_name) {
        if let Some(loc) = storage.location {
            return Some(loc);
        }
    }
    None
}

pub fn class_constant_exists_in_chain(db: &dyn MirDatabase, fqcn: &str, const_name: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    crate::db::find_class_constant_in_chain(db, here, const_name).is_some()
}

/// Subtype check: does `child` extend or implement `ancestor`?
///
/// Run on every assignment / arg / return / throw-catch type-compatibility
/// check — by far the hottest consumer of FQCN interning (`ustr::Ustr::from`
/// takes a global lock). The same `(child, ancestor)` pairs recur constantly,
/// so when a **pass-scoped** subtype cache is present (set on the batch body
/// pass via `freeze_workspace_index`) we hash the raw `&str` pair and return
/// **before** any interning. The cache is sound for the same reason the frozen
/// index is: the class graph is immutable for the duration of the pass, so a
/// memoized answer can't go stale. On the canonical / open-file db the cache is
/// absent (`None`) and every call recomputes — correct under mid-analysis
/// mutation, just not accelerated.
pub fn extends_or_implements(db: &dyn MirDatabase, child: &str, ancestor: &str) -> bool {
    if child == ancestor {
        return true;
    }
    let Some(cache) = db.subtype_cache() else {
        return extends_or_implements_uncached(db, child, ancestor);
    };
    use std::hash::{Hash, Hasher};
    let mut h1 = rustc_hash::FxHasher::default();
    child.hash(&mut h1);
    let mut h2 = rustc_hash::FxHasher::default();
    ancestor.hash(&mut h2);
    let key = (h1.finish(), h2.finish());
    // Collision-safe: the stored strings are verified against the lookup pair,
    // so a hash collision degrades to a recompute, never a wrong answer.
    if let Some(entry) = cache.get(&key) {
        if entry.0.as_ref() == child && entry.1.as_ref() == ancestor {
            return entry.2;
        }
    }
    let result = extends_or_implements_uncached(db, child, ancestor);
    // `or_insert_with` so a colliding entry (different pair, same hash) is left
    // intact — that pair just won't be cached.
    cache
        .entry(key)
        .or_insert_with(|| (Box::from(child), Box::from(ancestor), result));
    result
}

fn extends_or_implements_uncached(db: &dyn MirDatabase, child: &str, ancestor: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, child);
    let Some(class) = crate::db::find_class_like(db, here) else {
        return false;
    };

    // If the ancestor is namespace-qualified but doesn't exist in the symbol table,
    // fall back to the short name. This recovers global stub classes (DOMNode, etc.)
    // that were incorrectly namespace-qualified when referenced from namespaced files
    // without a leading backslash.
    let short: Option<&str> = if ancestor.contains('\\') {
        let fqcn = crate::db::Fqcn::from_str(db, ancestor);
        if crate::db::find_class_like(db, fqcn).is_none() {
            ancestor.rsplit('\\').next()
        } else {
            None
        }
    } else {
        None
    };
    let eff = short.unwrap_or(ancestor);

    if child == eff {
        return true;
    }

    if class.is_enum() {
        if class.interfaces().iter().any(|i| i.as_ref() == eff) {
            return true;
        }
        if eff == "UnitEnum" || eff == "\\UnitEnum" {
            return true;
        }
        if (eff == "BackedEnum" || eff == "\\BackedEnum") && class.is_backed_enum() {
            return true;
        }
        return false;
    }
    crate::db::class_ancestors_by_fqcn(db, here)
        .iter()
        .any(|p| p.as_ref() == eff)
}

// parse_file tracked query (S0 — owned parse, salsa-memoized)

/// Newtype so `Arc<ParseResult>` can be a salsa tracked-query return type.
///
/// Equality is pointer identity — salsa uses it to decide whether
/// downstream queries need re-running after a re-parse.
#[derive(Clone)]
pub struct TrackedParseResult(pub Arc<php_rs_parser::ParseResult>);

impl std::fmt::Debug for TrackedParseResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("TrackedParseResult").finish()
    }
}

impl PartialEq for TrackedParseResult {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for TrackedParseResult {}

/// Parse `file`'s source text into a fully-owned, lifetime-free
/// [`php_rs_parser::ParseResult`] and memoize it in Salsa.
///
/// Salsa invalidates this memo whenever `file.text(db)` changes.  All
/// downstream queries that need the AST or source-map should call this
/// instead of calling `php_rs_parser::parse` directly, so they get the
/// cached result for free on every incremental edit that doesn't touch
/// this file.
///
/// `lru = 256` caps the number of owned ASTs held in memory. Beyond that
/// cap, the least-recently-used parse is evicted; if a later query re-asks,
/// salsa re-parses on demand. For CLI cold-start the visit pattern is
/// "parse once, never re-ask," so eviction is effectively free; for LSP
/// the working set is small. Without the cap, parsing 12k files would
/// pin ~1–4 GB of owned AST for the whole session.
#[salsa::tracked(lru = 256)]
pub fn parse_file(db: &dyn MirDatabase, file: SourceFile) -> TrackedParseResult {
    let text = file.text(db);
    TrackedParseResult(Arc::new(php_rs_parser::parse(text.as_ref())))
}

// collect_file_definitions tracked query (S1)

/// Uncached version of collect_file_definitions for bulk operations like vendor
/// collection, where we don't need Salsa to cache the intermediate StubSlice
/// results. This avoids holding Arc<StubSlice> in Salsa's query cache after
/// ingestion.
pub fn collect_file_definitions_uncached(
    db: &dyn MirDatabase,
    file: SourceFile,
) -> FileDefinitions {
    let path = file.path(db);
    let text = file.text(db);

    let php_version = db_php_version(db);

    // Content hash needed for both in-process and disk cache lookups.
    let content_hash = crate::stub_cache::hash_source(text);

    // Fast path 1: in-process parse cache (populated by collect_and_ingest_file).
    // Avoids re-parsing files that were already processed in the same session.
    // Safe inside a tracked query: content-addressed by source hash, not mutated.
    if let Some(cached) = db
        .parse_cache()
        .get(&content_hash, php_version.cache_byte())
    {
        crate::metrics::record_stub_cache_hit();
        if cached.file.as_deref() == Some(path) {
            // Path matches — share the Arc directly (no data clone needed).
            return FileDefinitions {
                slice: cached,
                issues: Arc::new(Vec::new()),
            };
        }
        // Different path — same source text at a different location.
        // Must fix the `file` field before returning.
        let mut owned = (*cached).clone();
        owned.file = Some(path.clone());
        crate::stub_cache::prepare_for_ingest(&mut owned);
        return FileDefinitions {
            slice: Arc::new(owned),
            issues: Arc::new(Vec::new()),
        };
    }

    // Fast path 2: disk cache hit avoids arena alloc + parse + collection walk.
    // Safe inside a tracked query because the cache is content-addressed.
    let disk_cache_state = db.stub_cache().map(|cache| {
        let php_v = php_version.cache_byte();
        (cache, php_v)
    });
    if let Some((cache, php_v)) = &disk_cache_state {
        if let Some(mut slice) = cache.get(path, &content_hash, *php_v) {
            crate::stub_cache::prepare_for_ingest(&mut slice);
            crate::metrics::record_stub_cache_hit();
            return FileDefinitions {
                slice: Arc::new(slice),
                issues: Arc::new(Vec::new()),
            };
        }
        crate::metrics::record_stub_cache_miss();
    }

    let parsed = php_rs_parser::parse(text);

    let has_hard_parse_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);

    let mut all_issues: Vec<Issue> = parsed
        .errors
        .iter()
        .filter(|err| !crate::parser::is_spurious_reserved_class_error(err))
        .map(|err| crate::parser::parse_error_to_issue(err, path, text, &parsed.source_map))
        .collect();

    let collector = crate::collector::DefinitionCollector::new_for_slice(
        path.clone(),
        text,
        &parsed.source_map,
    )
    .with_php_version(php_version);
    let (mut slice, collector_issues) = collector.collect_slice(&parsed.program);
    all_issues.extend(collector_issues);
    mir_codebase::definitions::deduplicate_params_in_slice(&mut slice);

    let slice_arc = Arc::new(slice);

    // Write back to both caches as long as the AST parsed cleanly.
    //
    // Collector-emitted diagnostics (malformed docblocks, unknown @psalm
    // annotations, etc.) are warnings on a fully-parsed AST — the slice is
    // complete and valid. Excluding those files from the cache forces a
    // re-parse on every subsequent session for any vendor file with so much
    // as a docblock warning. Only hard parse errors (incomplete AST) block
    // caching.
    if !has_hard_parse_errors {
        // In-process cache: prevents re-parsing in the same session.
        db.parse_cache().insert(
            content_hash,
            php_version.cache_byte(),
            Arc::clone(&slice_arc),
        );
        // Disk cache: prevents re-parsing in future sessions.
        if let Some((cache, php_v)) = &disk_cache_state {
            cache.put(path, &content_hash, *php_v, &slice_arc);
        }
    }

    FileDefinitions {
        slice: slice_arc,
        issues: Arc::new(all_issues),
    }
}

/// `lru = 4096` caps the number of `FileDefinitions` results held in Salsa's
/// memo table. Without an LRU, every vendor class file ever loaded in a
/// long-running LSP session retains its `Arc<StubSlice>` in the memo
/// indefinitely, even after the file is evicted from the session via
/// `invalidate_file`. 4096 safely exceeds the typical simultaneous active-file
/// count (stubs ≈120 + vendor hundreds + workspace hundreds) so active files
/// are never churned; vendor files loaded and later evicted are displaced by
/// subsequent queries, freeing their StubSlice allocations.
#[salsa::tracked(lru = 4096)]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    collect_file_definitions_uncached(db, file)
}

// File-level inferred-type Salsa query

type MethodInferMap = FxHashMap<(Arc<str>, Arc<str>), Arc<Type>>;
// Property names are case-sensitive in PHP (unlike method names), so unlike
// `MethodInferMap` this is keyed by the name as written, not lowercased.
type PropertyInferMap = FxHashMap<(Arc<str>, Arc<str>), Arc<Type>>;

#[derive(Clone, Debug)]
pub struct InferredFileTypes {
    pub functions: Arc<FxHashMap<Arc<str>, Arc<Type>>>,
    pub methods: Arc<MethodInferMap>,
    pub properties: Arc<PropertyInferMap>,
}

impl InferredFileTypes {
    pub fn empty() -> Self {
        Self {
            functions: Arc::new(FxHashMap::default()),
            methods: Arc::new(MethodInferMap::default()),
            properties: Arc::new(PropertyInferMap::default()),
        }
    }
}

impl PartialEq for InferredFileTypes {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.methods, &other.methods)
            && Arc::ptr_eq(&self.properties, &other.properties)
        {
            return true;
        }
        if self.functions.len() != other.functions.len()
            || self.methods.len() != other.methods.len()
            || self.properties.len() != other.properties.len()
        {
            return false;
        }
        for (k, v) in self.functions.iter() {
            if other.functions.get(k).is_none_or(|ov| *ov != *v) {
                return false;
            }
        }
        for (k, v) in self.methods.iter() {
            if other.methods.get(k).is_none_or(|ov| *ov != *v) {
                return false;
            }
        }
        for (k, v) in self.properties.iter() {
            if other.properties.get(k).is_none_or(|ov| *ov != *v) {
                return false;
            }
        }
        true
    }
}

fn infer_file_return_types_initial(
    _db: &dyn MirDatabase,
    _id: salsa::Id,
    _file: SourceFile,
) -> InferredFileTypes {
    InferredFileTypes::empty()
}

fn infer_file_return_types_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &InferredFileTypes,
    _value: InferredFileTypes,
    _file: SourceFile,
) -> InferredFileTypes {
    InferredFileTypes::empty()
}

/// Memoized parse of the configured PHP version string.
///
/// Reads the `AnalyzeFileInput.php_version` salsa input so that salsa tracks
/// the dependency correctly; `from_str` is called at most once per version
/// change instead of once per file.
#[salsa::tracked(returns(copy))]
pub fn db_php_version(db: &dyn MirDatabase) -> crate::php_version::PhpVersion {
    use std::str::FromStr as _;
    crate::php_version::PhpVersion::from_str(db.analyze_config().php_version(db).as_ref())
        .unwrap_or(crate::php_version::PhpVersion::LATEST)
}

#[salsa::tracked(cycle_fn = infer_file_return_types_cycle, cycle_initial = infer_file_return_types_initial)]
pub fn infer_file_return_types(db: &dyn MirDatabase, file: SourceFile) -> InferredFileTypes {
    let path = file.path(db);
    let text = file.text(db);
    let php_version = db_php_version(db);

    let parsed_file = parse_file(db, file);
    let parsed = &*parsed_file.0;

    if parsed.errors.iter().any(crate::parser::is_hard_parse_error) {
        return InferredFileTypes::empty();
    }

    let driver = crate::body_analysis::BodyAnalyzer::new_inference_only(db, php_version);
    driver.analyze_bodies(
        &parsed.program,
        path.clone(),
        text.as_ref(),
        &parsed.source_map,
    );
    let inferred = driver.take_inferred_types();

    let mut functions: FxHashMap<Arc<str>, Arc<Type>> =
        FxHashMap::with_capacity_and_hasher(inferred.functions.len(), Default::default());
    for (fqn, ty) in inferred.functions {
        functions.insert(fqn, mir_codebase::definitions::wrap_var_type(ty));
    }

    let mut methods: FxHashMap<(Arc<str>, Arc<str>), Arc<Type>> =
        FxHashMap::with_capacity_and_hasher(inferred.methods.len(), Default::default());
    for (fqcn, name, ty) in inferred.methods {
        let name_lower: Arc<str> = if name.bytes().any(|b| b.is_ascii_uppercase()) {
            Arc::from(crate::util::php_ident_lowercase(&name).as_str())
        } else {
            name
        };
        methods.insert(
            (fqcn, name_lower),
            mir_codebase::definitions::wrap_var_type(ty),
        );
    }

    let mut properties: FxHashMap<(Arc<str>, Arc<str>), Arc<Type>> =
        FxHashMap::with_capacity_and_hasher(inferred.properties.len(), Default::default());
    for (fqcn, name, ty) in inferred.properties {
        // Multiple constructor assignment sites for the same property merge
        // via union, same reasoning as `merge_return_types` for return
        // statements — any of them could be the runtime value.
        properties
            .entry((fqcn, name))
            .and_modify(|existing: &mut Arc<Type>| {
                let mut merged = (**existing).clone();
                merged.merge_with(&ty);
                *existing = mir_codebase::definitions::wrap_var_type(merged);
            })
            .or_insert_with(|| mir_codebase::definitions::wrap_var_type(ty));
    }

    InferredFileTypes {
        functions: Arc::new(functions),
        methods: Arc::new(methods),
        properties: Arc::new(properties),
    }
}

pub fn is_unchecked_exception(db: &dyn MirDatabase, fqcn: &str) -> bool {
    extends_or_implements(db, fqcn, "RuntimeException")
        || extends_or_implements(db, fqcn, "LogicException")
}
