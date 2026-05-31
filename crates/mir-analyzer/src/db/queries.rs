use std::sync::Arc;

use mir_codebase::storage::TemplateParam;
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
        if let Some(imports) = (!name.starts_with('\\')).then(|| db.file_imports(file)) {
            if let Some((first, rest)) = name.split_once('\\') {
                if let Some(base) = imports.get(&Name::new(first)) {
                    return format!("{}\\{rest}", base.as_str());
                }
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

    let imports = db.file_imports(file);
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

pub fn class_template_params(db: &dyn MirDatabase, fqcn: &str) -> Option<Arc<[TemplateParam]>> {
    let here = crate::db::Fqcn::from_str(db, fqcn);
    let class = crate::db::find_class_like(db, here)?;
    Some(Arc::from(class.template_params().to_vec()))
}

/// Walk the parent chain collecting template bindings from `@extends` type
/// args. For `class UserRepo extends BaseRepo` with `@extends BaseRepo<User>`,
/// returns `{ T → User }` where `T` is `BaseRepo`'s declared template parameter.
pub fn inherited_template_bindings(db: &dyn MirDatabase, fqcn: &str) -> FxHashMap<Name, Type> {
    let mut bindings: FxHashMap<Name, Type> = FxHashMap::default();
    let mut visited: FxHashSet<Arc<str>> = FxHashSet::default();
    let mut current: Arc<str> = Arc::from(fqcn);
    loop {
        if !visited.insert(current.clone()) {
            break;
        }
        let Some(class) =
            crate::db::find_class_like(db, crate::db::Fqcn::from_str(db, current.as_ref()))
        else {
            break;
        };
        let Some(parent) = class.parent().cloned() else {
            break;
        };
        let extends_type_args = class.extends_type_args();
        if !extends_type_args.is_empty() {
            if let Some(parent_tps) = class_template_params(db, parent.as_ref()) {
                for (tp, ty) in parent_tps.iter().zip(extends_type_args.iter()) {
                    bindings.entry(tp.name).or_insert_with(|| ty.clone());
                }
            }
        }
        current = parent;
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
    if let Some((_, storage)) = crate::db::find_method_in_chain(db, here, member_name) {
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

pub fn extends_or_implements(db: &dyn MirDatabase, child: &str, ancestor: &str) -> bool {
    if child == ancestor {
        return true;
    }
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

unsafe impl salsa::Update for TrackedParseResult {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
        true
    }
}

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

    use std::str::FromStr as _;
    let php_version = crate::php_version::PhpVersion::from_str(db.php_version_str().as_ref())
        .unwrap_or(crate::php_version::PhpVersion::LATEST);

    // Content hash needed for both in-process and disk cache lookups.
    let content_hash = crate::stub_cache::hash_source(&text);

    // Fast path 1: in-process parse cache (populated by collect_and_ingest_file).
    // Avoids re-parsing files that were already processed in the same session.
    // Safe inside a tracked query: content-addressed by source hash, not mutated.
    if let Some(cached) = db.parse_cache().get(&content_hash).map(|r| Arc::clone(&*r)) {
        crate::metrics::record_stub_cache_hit();
        if cached.file.as_deref() == Some(&*path) {
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
        if let Some(mut slice) = cache.get(&path, &content_hash, *php_v) {
            crate::stub_cache::prepare_for_ingest(&mut slice);
            crate::metrics::record_stub_cache_hit();
            return FileDefinitions {
                slice: Arc::new(slice),
                issues: Arc::new(Vec::new()),
            };
        }
        crate::metrics::record_stub_cache_miss();
    }

    let parsed = php_rs_parser::parse(&text);

    let has_hard_parse_errors = parsed.errors.iter().any(crate::parser::is_hard_parse_error);

    let mut all_issues: Vec<Issue> = parsed
        .errors
        .iter()
        .map(|err| crate::parser::parse_error_to_issue(err, &path, &text, &parsed.source_map))
        .collect();

    let collector = crate::collector::DefinitionCollector::new_for_slice(
        path.clone(),
        &text,
        &parsed.source_map,
    )
    .with_php_version(php_version);
    let (mut slice, collector_issues) = collector.collect_slice(&parsed.program);
    all_issues.extend(collector_issues);
    mir_codebase::storage::deduplicate_params_in_slice(&mut slice);

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
        db.parse_cache()
            .insert(content_hash, Arc::clone(&slice_arc));
        // Disk cache: prevents re-parsing in future sessions.
        if let Some((cache, php_v)) = &disk_cache_state {
            cache.put(&path, &content_hash, *php_v, &slice_arc);
        }
    }

    FileDefinitions {
        slice: slice_arc,
        issues: Arc::new(all_issues),
    }
}

#[salsa::tracked]
pub fn collect_file_definitions(db: &dyn MirDatabase, file: SourceFile) -> FileDefinitions {
    collect_file_definitions_uncached(db, file)
}

// File-level inferred-type Salsa query

type MethodInferMap = FxHashMap<(Arc<str>, Arc<str>), Arc<Type>>;

#[derive(Clone, Debug)]
pub struct InferredFileTypes {
    pub functions: Arc<FxHashMap<Arc<str>, Arc<Type>>>,
    pub methods: Arc<MethodInferMap>,
}

impl InferredFileTypes {
    pub fn empty() -> Self {
        Self {
            functions: Arc::new(FxHashMap::default()),
            methods: Arc::new(MethodInferMap::default()),
        }
    }
}

impl PartialEq for InferredFileTypes {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.functions, &other.functions)
            && Arc::ptr_eq(&self.methods, &other.methods)
        {
            return true;
        }
        if self.functions.len() != other.functions.len()
            || self.methods.len() != other.methods.len()
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
        true
    }
}

unsafe impl salsa::Update for InferredFileTypes {
    unsafe fn maybe_update(old_ptr: *mut Self, new_val: Self) -> bool {
        let old = unsafe { &mut *old_ptr };
        if *old == new_val {
            return false;
        }
        *old = new_val;
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

#[salsa::tracked(cycle_fn = infer_file_return_types_cycle, cycle_initial = infer_file_return_types_initial)]
pub fn infer_file_return_types(db: &dyn MirDatabase, file: SourceFile) -> InferredFileTypes {
    use std::str::FromStr as _;
    let path = file.path(db);
    let text = file.text(db);
    let php_version = crate::php_version::PhpVersion::from_str(db.php_version_str().as_ref())
        .unwrap_or(crate::php_version::PhpVersion::LATEST);

    let parsed_file = parse_file(db, file);
    let parsed = &*parsed_file.0;

    if parsed.errors.iter().any(crate::parser::is_hard_parse_error) {
        return InferredFileTypes::empty();
    }

    let driver = crate::body_analysis::BodyAnalyzer::new_inference_only(db, php_version);
    driver.analyze_bodies(&parsed.program, path, text.as_ref(), &parsed.source_map);
    let inferred = driver.take_inferred_types();

    let mut functions: FxHashMap<Arc<str>, Arc<Type>> =
        FxHashMap::with_capacity_and_hasher(inferred.functions.len(), Default::default());
    for (fqn, ty) in inferred.functions {
        functions.insert(fqn, Arc::new(ty));
    }

    let mut methods: FxHashMap<(Arc<str>, Arc<str>), Arc<Type>> =
        FxHashMap::with_capacity_and_hasher(inferred.methods.len(), Default::default());
    for (fqcn, name, ty) in inferred.methods {
        let name_lower: Arc<str> = if name.chars().all(|c| !c.is_uppercase()) {
            name
        } else {
            Arc::from(name.to_lowercase().as_str())
        };
        methods.insert((fqcn, name_lower), Arc::new(ty));
    }

    InferredFileTypes {
        functions: Arc::new(functions),
        methods: Arc::new(methods),
    }
}

pub fn is_unchecked_exception(db: &dyn MirDatabase, fqcn: &str) -> bool {
    extends_or_implements(db, fqcn, "RuntimeException")
        || extends_or_implements(db, fqcn, "LogicException")
}
