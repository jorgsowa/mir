use std::sync::Arc;

use mir_codebase::storage::{Location, TemplateParam};
use mir_issues::Issue;
use mir_types::Union;
use rustc_hash::FxHashMap;

use super::*;

/// Snapshot of a class's discriminator + abstractness.
#[derive(Debug, Clone, Copy)]
pub struct ClassKind {
    pub is_interface: bool,
    pub is_trait: bool,
    pub is_enum: bool,
    pub is_abstract: bool,
}

pub fn class_kind_via_db(db: &dyn MirDatabase, fqcn: &str) -> Option<ClassKind> {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqcn));
    let class = crate::db::find_class_like(db, here)?;
    Some(ClassKind {
        is_interface: class.is_interface(),
        is_trait: class.is_trait(),
        is_enum: class.is_enum(),
        is_abstract: class.is_abstract(),
    })
}

pub fn type_exists_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqcn));
    crate::db::find_class_like(db, here).is_some()
}

#[allow(dead_code)]
pub fn function_exists_via_db(db: &dyn MirDatabase, fqn: &str) -> bool {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqn));
    crate::db::find_function(db, here).is_some()
}

#[allow(dead_code)]
pub fn constant_exists_via_db(db: &dyn MirDatabase, fqn: &str) -> bool {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqn));
    crate::db::find_global_constant(db, here).is_some()
}

pub fn resolve_name_via_db(db: &dyn MirDatabase, file: &str, name: &str) -> String {
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
                if let Some(base) = imports.get(first) {
                    return format!("{base}\\{rest}");
                }
            }
        }
        // If the name is already a known FQCN (e.g. stored in TNamedObject by a prior
        // resolution step), return it unchanged to avoid double-prepending the namespace.
        if type_exists_via_db(db, name) {
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
    if let Some(fqcn) = imports.get(name) {
        return fqcn.clone();
    }
    if let Some((_, fqcn)) = imports
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
    {
        return fqcn.clone();
    }
    if let Some(ns) = db.file_namespace(file) {
        return format!("{}\\{}", ns, name);
    }
    name.to_string()
}

pub fn class_template_params_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
) -> Option<Arc<[TemplateParam]>> {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqcn));
    let class = crate::db::find_class_like(db, here)?;
    Some(Arc::from(class.template_params().to_vec()))
}

/// Walk the parent chain collecting template bindings from `@extends` type
/// args. For `class UserRepo extends BaseRepo` with `@extends BaseRepo<User>`,
/// returns `{ T → User }` where `T` is `BaseRepo`'s declared template parameter.
pub fn inherited_template_bindings_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
) -> std::collections::HashMap<Arc<str>, Union> {
    let mut bindings: std::collections::HashMap<Arc<str>, Union> = std::collections::HashMap::new();
    let mut visited: rustc_hash::FxHashSet<Arc<str>> = rustc_hash::FxHashSet::default();
    let mut current: Arc<str> = Arc::from(fqcn);
    loop {
        if !visited.insert(current.clone()) {
            break;
        }
        let Some(class) = crate::db::find_class_like(db, crate::db::Fqcn::new(db, current.clone()))
        else {
            break;
        };
        let Some(parent) = class.parent().cloned() else {
            break;
        };
        let extends_type_args = class.extends_type_args();
        if !extends_type_args.is_empty() {
            if let Some(parent_tps) = class_template_params_via_db(db, parent.as_ref()) {
                for (tp, ty) in parent_tps.iter().zip(extends_type_args.iter()) {
                    bindings
                        .entry(tp.name.clone())
                        .or_insert_with(|| ty.clone());
                }
            }
        }
        current = parent;
    }
    bindings
}

pub fn has_unknown_ancestor_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqcn));
    if crate::db::find_class_like(db, here).is_none() {
        return false;
    }
    crate::db::class_ancestors_by_fqcn(db, here)
        .iter()
        .skip(1) // self
        .any(|ancestor| !type_exists_via_db(db, ancestor))
}

pub fn method_is_concretely_implemented(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> bool {
    crate::db::is_method_concretely_implemented_pull(db, fqcn, method_name)
}

pub fn member_location_via_db(
    db: &dyn MirDatabase,
    fqcn: &str,
    member_name: &str,
) -> Option<Location> {
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqcn));
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
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqcn));
    crate::db::find_class_constant_in_chain(db, here, const_name).is_some()
}

pub fn extends_or_implements_via_db(db: &dyn MirDatabase, child: &str, ancestor: &str) -> bool {
    if child == ancestor {
        return true;
    }
    let here = crate::db::Fqcn::new(db, Arc::<str>::from(child));
    let Some(class) = crate::db::find_class_like(db, here) else {
        return false;
    };
    if class.is_enum() {
        if class.interfaces().iter().any(|i| i.as_ref() == ancestor) {
            return true;
        }
        if ancestor == "UnitEnum" || ancestor == "\\UnitEnum" {
            return true;
        }
        if (ancestor == "BackedEnum" || ancestor == "\\BackedEnum") && class.is_backed_enum() {
            return true;
        }
        return false;
    }
    crate::db::class_ancestors_by_fqcn(db, here)
        .iter()
        .any(|p| p.as_ref() == ancestor)
}

// parse_file tracked query (S0 — owned parse, salsa-memoized)

/// Newtype so `Arc<ParseResult>` can be a salsa tracked-query return type.
///
/// Equality is pointer identity — salsa uses it to decide whether
/// downstream queries need re-running after a re-parse.
#[derive(Clone)]
pub struct ParsedFile(pub Arc<php_rs_parser::ParseResult>);

impl std::fmt::Debug for ParsedFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ParsedFile").finish()
    }
}

impl PartialEq for ParsedFile {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ParsedFile {}

unsafe impl salsa::Update for ParsedFile {
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
#[salsa::tracked]
pub fn parse_file(db: &dyn MirDatabase, file: SourceFile) -> ParsedFile {
    let text = file.text(db);
    ParsedFile(Arc::new(php_rs_parser::parse(text.as_ref())))
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
    let has_collector_issues = !collector_issues.is_empty();
    all_issues.extend(collector_issues);
    mir_codebase::storage::deduplicate_params_in_slice(&mut slice);

    let slice_arc = Arc::new(slice);

    // Write back to both caches on a clean parse.
    if !has_hard_parse_errors && !has_collector_issues {
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

type MethodInferMap = FxHashMap<(Arc<str>, Arc<str>), Arc<Union>>;

#[derive(Clone, Debug)]
pub struct InferredFileTypes {
    pub functions: Arc<FxHashMap<Arc<str>, Arc<Union>>>,
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

#[salsa::tracked]
pub fn infer_file_return_types(db: &dyn MirDatabase, file: SourceFile) -> InferredFileTypes {
    use std::str::FromStr as _;
    let path = file.path(db);
    let text = file.text(db);
    let php_version = crate::php_version::PhpVersion::from_str(db.php_version_str().as_ref())
        .unwrap_or(crate::php_version::PhpVersion::LATEST);

    let parsed = php_rs_parser::parse(text.as_ref());

    if parsed.errors.iter().any(crate::parser::is_hard_parse_error) {
        return InferredFileTypes::empty();
    }

    let driver = crate::pass2::Pass2Driver::new_inference_only(db, php_version);
    driver.analyze_bodies(&parsed.program, path, text.as_ref(), &parsed.source_map);
    let inferred = driver.take_inferred_types();

    let mut functions: FxHashMap<Arc<str>, Arc<Union>> =
        FxHashMap::with_capacity_and_hasher(inferred.functions.len(), Default::default());
    for (fqn, ty) in inferred.functions {
        functions.insert(fqn, Arc::new(ty));
    }

    let mut methods: FxHashMap<(Arc<str>, Arc<str>), Arc<Union>> =
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

#[allow(dead_code)]
pub(crate) fn collect_accumulated_issues(
    db: &dyn MirDatabase,
    files: &[(Arc<str>, SourceFile)],
    php_version: &str,
) -> Vec<Issue> {
    let mut all_issues = Vec::new();
    let input = AnalyzeFileInput::new(db, Arc::from(php_version));

    for (_path, file) in files {
        analyze_file(db, *file, input);
        let accumulated: Vec<&IssueAccumulator> = analyze_file::accumulated(db, *file, input);
        for acc in accumulated {
            all_issues.push(acc.0.clone());
        }
    }

    all_issues
}

pub fn is_unchecked_exception_via_db(db: &dyn MirDatabase, fqcn: &str) -> bool {
    extends_or_implements_via_db(db, fqcn, "RuntimeException")
        || extends_or_implements_via_db(db, fqcn, "LogicException")
}
