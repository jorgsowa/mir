//! Batch-oriented project analysis on [`AnalysisSession`].
//!
//! This module hosts the multi-file orchestration that used to live on the
//! retired `ProjectAnalyzer`: parallel definition collection, lazy class loading, dead-code
//! sweep, reverse-dependency index, and the [`AnalysisResult`] return type.
//! Per-file (LSP) entry points stay on `AnalysisSession` itself in
//! `session.rs`.
//!
//! All methods are `impl AnalysisSession`; configuration that's only
//! meaningful for batch runs (issue suppressions, progress callback, optional
//! PHP version override) is grouped in [`BatchOptions`] and passed in rather
//! than stored on the session.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use rayon::prelude::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use mir_issues::Issue;
use mir_types::{Atomic, Type};

use crate::body_analysis::BodyAnalyzer;
use crate::cache::{hash_content, surface_fingerprint};
use crate::db::{
    collect_file_definitions, FileDefinitions, MirDatabase, MirDbStorage, RefLoc, SourceFile,
};
use crate::php_version::PhpVersion;
use crate::session::AnalysisSession;
use crate::stub_cache::{hash_source, prepare_for_ingest};

/// Issue kinds emitted by [`crate::dead_code::DeadCodeAnalyzer`].
///
/// The dead-code pass is just an error group — these names participate in
/// [`BatchOptions::suppressed_issue_kinds`] like any other `IssueKind`. If
/// every kind listed here is suppressed, the dead-code pass is skipped
/// entirely.
pub fn dead_code_issue_kinds() -> &'static [&'static str] {
    &[
        "UnusedMethod",
        "UnusedProperty",
        "UnusedFunction",
        "UnusedClass",
    ]
}

/// Per-batch options for [`AnalysisSession::analyze_paths`] and friends.
///
/// Configuration that only makes sense for full-project (batch) analysis
/// lives here instead of on [`AnalysisSession`], so the per-file LSP API
/// isn't bloated with state nothing else reads.
#[derive(Clone, Default)]
pub struct BatchOptions {
    /// Names of `IssueKind` variants to drop from the final result, e.g.
    /// `["MissingThrowsDocblock", "UnusedMethod"]`. Applied as a final
    /// post-filter so analyzer internals don't need to know which
    /// diagnostics the consumer cares about. Empty by default.
    pub suppressed_issue_kinds: HashSet<String>,
    /// Called once after each file completes body analysis (progress reporting).
    pub on_file_done: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Override the session's configured PHP version for this run. `None`
    /// uses the session's version.
    pub php_version_override: Option<PhpVersion>,
    /// Skip collecting per-expression [`crate::symbol::ResolvedSymbol`]s
    /// into the [`AnalysisResult`]. Defaults to `false` (symbols collected)
    /// so existing consumers — LSP servers using
    /// [`AnalysisResult::symbol_at`] for hover/go-to-definition — are
    /// unaffected. Diagnostics-only consumers (the CLI) opt out: a
    /// Laravel-scale batch retains ~600k symbols nothing reads.
    pub skip_symbols: bool,
}

impl BatchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_suppressed<I, S>(mut self, kinds: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.suppressed_issue_kinds = kinds.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_progress_callback(mut self, callback: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_file_done = Some(callback);
        self
    }

    pub fn with_php_version(mut self, version: PhpVersion) -> Self {
        self.php_version_override = Some(version);
        self
    }

    /// Don't collect per-expression symbols into the result (see
    /// [`Self::skip_symbols`]). For diagnostics-only consumers;
    /// [`AnalysisResult::symbol_at`] will find nothing on the batch result.
    pub fn without_symbols(mut self) -> Self {
        self.skip_symbols = true;
        self
    }

    /// True iff at least one dead-code [`IssueKind`] would be emitted (i.e.
    /// not all of them are suppressed).
    fn should_run_dead_code(&self) -> bool {
        dead_code_issue_kinds()
            .iter()
            .any(|k| !self.suppressed_issue_kinds.contains(*k))
    }

    /// Drop issues whose [`IssueKind::display_name()`] is listed in
    /// [`Self::suppressed_issue_kinds`] (display_name so plugin issues can be
    /// suppressed by their own names).
    fn apply(&self, issues: &mut Vec<Issue>) {
        if self.suppressed_issue_kinds.is_empty() {
            return;
        }
        issues.retain(|i| !self.suppressed_issue_kinds.contains(i.kind.display_name()));
    }
}

struct ParsedProjectFile {
    file: Arc<str>,
    source: Arc<str>,
    parsed: php_rs_parser::ParseResult,
}

impl ParsedProjectFile {
    fn new(file: Arc<str>, source: Arc<str>) -> Self {
        let parsed = php_rs_parser::parse(source.as_ref());
        Self {
            file,
            source,
            parsed,
        }
    }

    fn source(&self) -> &str {
        self.source.as_ref()
    }

    fn source_map(&self) -> &php_rs_parser::source_map::SourceMap {
        &self.parsed.source_map
    }

    fn errors(&self) -> &[php_rs_parser::diagnostics::ParseError] {
        &self.parsed.errors
    }

    fn owned(&self) -> &php_ast::owned::Program {
        &self.parsed.program
    }
}

impl AnalysisSession {
    /// Cumulative hit / miss counts on the persistent definition cache attached
    /// to this session. `(0, 0)` when no cache is configured.
    #[doc(hidden)]
    pub fn stub_cache_stats(&self) -> (u64, u64) {
        match self.db.stub_cache.as_deref() {
            Some(c) => (c.hits(), c.misses()),
            None => (0, 0),
        }
    }

    fn batch_php_version(&self, opts: &BatchOptions) -> PhpVersion {
        opts.php_version_override.unwrap_or(self.php_version)
    }

    /// Mark issues silenced by inline suppression comments
    /// (`@mir-ignore`, `@psalm-suppress`, `@phpstan-ignore*`, …) as suppressed.
    ///
    /// Runs as a final post-filter over the merged issue list so it applies
    /// uniformly to every emitting pass — body analysis, the collector, class
    /// checks and dead-code detection — including diagnostics the per-statement
    /// `@psalm-suppress` path in `stmt/mod.rs` structurally cannot reach.
    ///
    /// Issues are *marked* rather than dropped, mirroring that per-statement
    /// path and the kind-level `mir.xml` suppress handler; every consumer (CLI,
    /// WASM, the test harness) already skips [`Issue::suppressed`].
    /// Apply inline suppressions and then emit `UnusedSuppress` issues for
    /// any named `@suppress`/`@psalm-suppress` annotations that matched nothing.
    ///
    /// `analyzed_files` must list every file that was analyzed in this batch so
    /// that files with *zero* existing issues still have their suppression maps
    /// inspected for unused annotations.
    fn apply_suppressions_and_emit_unused(
        &self,
        issues: &mut Vec<Issue>,
        analyzed_files: &[Arc<str>],
    ) {
        use crate::suppression::SuppressionMap;
        let db = self.snapshot_db();
        let mut cache: HashMap<Arc<str>, Option<SuppressionMap>> = HashMap::default();
        for issue in issues.iter_mut() {
            if issue.suppressed {
                continue;
            }
            let map = cache.entry(issue.location.file.clone()).or_insert_with(|| {
                db.lookup_source_file(&issue.location.file)
                    .map(|sf| SuppressionMap::from_source(sf.text(&db)))
            });
            if let Some(map) = map.as_ref() {
                if map.is_suppressed(issue.location.line, issue.kind.display_name(), issue.kind.code()) {
                    issue.suppressed = true;
                }
            }
        }
        // Ensure suppression maps are built for every analyzed file, not just
        // those that already have at least one issue (files with no issues would
        // otherwise be skipped and their unused suppressions never detected).
        for file in analyzed_files {
            cache.entry(file.clone()).or_insert_with(|| {
                db.lookup_source_file(file)
                    .map(|sf| SuppressionMap::from_source(sf.text(&db)))
            });
        }
        // Now emit UnusedSuppress for each file that has named suppressions.
        let files: Vec<Arc<str>> = cache
            .iter()
            .filter_map(|(f, m)| m.as_ref().map(|_| f.clone()))
            .collect();
        // Bucket issue refs by file once — a per-file scan of the global
        // issue list would be O(files × issues), with clones on top.
        let mut issues_by_file: HashMap<&str, Vec<&Issue>> = HashMap::default();
        for issue in issues.iter() {
            issues_by_file
                .entry(issue.location.file.as_ref())
                .or_default()
                .push(issue);
        }
        let mut new_issues: Vec<Issue> = Vec::new();
        for file in files {
            if let Some(Some(map)) = cache.get(&file) {
                if map.named_suppressions.is_empty() {
                    continue;
                }
                let file_issues: &[&Issue] = issues_by_file
                    .get(file.as_ref())
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                // Pre-suppressed issues arrived with suppressed=true from the
                // IssueBuffer mechanism (collector / body analysis). They may be
                // at a different line than the SuppressionMap target and need
                // special handling in unused_named.
                let pre_suppressed: Vec<&Issue> = file_issues
                    .iter()
                    .filter(|i| i.suppressed)
                    .copied()
                    .collect();
                // Issues newly suppressed by the SuppressionMap in this pass
                // arrived with suppressed=false; after the marking loop they
                // also have suppressed=true. Pass all file issues for exact-line
                // matching; pre_suppressed enables the docblock-range fallback.
                let unused = map.unused_named(file_issues, &pre_suppressed);
                for (line, kind) in unused {
                    let loc = mir_types::Location::new(file.clone(), line, line, 0, 0);
                    let mut issue = Issue::new(mir_issues::IssueKind::UnusedSuppress { kind }, loc);
                    if map.is_suppressed(line, issue.kind.display_name(), issue.kind.code()) {
                        issue.suppressed = true;
                    }
                    new_issues.push(issue);
                }
            }
        }
        issues.extend(new_issues);
    }

    fn type_exists(&self, fqcn: &str) -> bool {
        let db = self.snapshot_db();
        crate::db::class_exists(&db, fqcn)
    }

    fn collect_and_ingest_source(
        &self,
        file: Arc<str>,
        src: &str,
        php_version: PhpVersion,
    ) -> FileDefinitions {
        self.db.collect_and_ingest_file(file, src, php_version)
    }

    /// Rebuild the workspace symbol index singleton from every registered source
    /// file. Required in the batch path because `workspace_index` reads the
    /// maintained singleton, and that singleton is built from vendor *before*
    /// `analyze_paths` registers project files (and before `lazy_load_*` faults
    /// in referenced classes). Without refreshing it, `find_class_like` /
    /// `class_exists` miss every project and lazy-loaded class, yielding false
    /// `UndefinedClass`. Cheap after the definition caches are warm (no parsing).
    fn refresh_workspace_index(&self) {
        let mut guard = self.db.salsa.write();
        guard.rebuild_workspace_symbol_index();
    }

    /// Load the configured PHP version + built-in stubs + user stubs into
    /// the shared db. Called by [`Self::analyze_paths`] and
    /// [`Self::collect_definitions`].
    fn load_batch_stubs(&self, php_version: PhpVersion) {
        // Wire the PHP version into the db before any SourceFile inputs are
        // registered — collect_file_definitions reads it for @since/@removed filtering.
        {
            let version_str = Arc::from(php_version.to_string().as_str());
            self.db.salsa.write().set_php_version(version_str);
        }

        // Built-in stubs for the configured PHP version.
        let paths: Vec<&'static str> = crate::stubs::stub_files().iter().map(|&(p, _)| p).collect();
        self.db.ingest_stub_paths(&paths, php_version);

        // User-configured stubs.
        self.db
            .ingest_user_stubs(&self.user_stub_files, &self.user_stub_dirs);

        // Ensure a resolver is configured so pull-path lookups can map
        // built-in FQCNs to the stub VFS paths registered above.
        let mut guard = self.db.salsa.write();
        if guard.current_resolver().is_none() {
            let resolver: Arc<dyn crate::ClassResolver> = Arc::new(crate::StubClassResolver);
            guard.set_resolver(Some(resolver));
        }
    }
}

mod lazy;
mod run;

/// Analyze a PHP source string without a real file path. Useful for tests
/// and single-file LSP mode. Allocates a throwaway db; doesn't touch any
/// existing session.
pub fn analyze_source(source: &str) -> AnalysisResult {
    let php_version = PhpVersion::LATEST;
    let file: Arc<str> = Arc::from("<source>");
    let mut db = MirDbStorage::default();
    db.set_php_version(Arc::from(php_version.to_string().as_str()));
    crate::stubs::load_stubs_for_version(&mut db, php_version);
    // Register the file through the workspace registry (not a bare
    // `SourceFile::new`) so it lands in `all_source_files()` and the
    // workspace symbol index. Without this, body analysis can't look up the
    // file's own functions/methods/classes and degrades every parameter to
    // `mixed` via the `ast_derived_fn_params` fallback.
    let salsa_file = db.upsert_source_file(file.clone(), Arc::from(source));
    let file_defs = collect_file_definitions(&db, salsa_file);
    let suppressions = crate::suppression::SuppressionMap::from_source(source);
    let mut all_issues = Arc::unwrap_or_clone(file_defs.issues.clone());
    if all_issues.iter().any(|issue| {
        matches!(issue.kind, mir_issues::IssueKind::ParseError { .. })
            && issue.severity == mir_issues::Severity::Error
    }) {
        mark_suppressed(&mut all_issues, &suppressions);
        return AnalysisResult::build(all_issues, rustc_hash::FxHashMap::default(), Vec::new());
    }
    let mut type_envs = rustc_hash::FxHashMap::default();
    let mut all_symbols = Vec::new();
    let result = php_rs_parser::parse(source);

    let driver = BodyAnalyzer::new(&db, php_version);
    all_issues.extend(driver.analyze_bodies_typed(
        &result.program,
        file.clone(),
        source,
        &result.source_map,
        &mut type_envs,
        &mut all_symbols,
    ));
    if let Some(plugins) = mir_plugin::snapshot() {
        if plugins.hooks().before_add_issue {
            all_issues.retain(|i| plugins.before_add_issue(i));
        }
    }
    mark_suppressed(&mut all_issues, &suppressions);
    emit_unused_suppressions(&mut all_issues, &suppressions, &file);
    AnalysisResult::build(all_issues, type_envs, all_symbols)
}

/// Mark issues silenced by a single file's [`SuppressionMap`]. Shared by the
/// in-memory [`analyze_source`] entry point, which has the source in hand and
/// does not go through the db-backed batch post-filter.
fn mark_suppressed(issues: &mut [Issue], suppressions: &crate::suppression::SuppressionMap) {
    if suppressions.is_empty() {
        return;
    }
    for issue in issues.iter_mut() {
        if !issue.suppressed
            && suppressions.is_suppressed(issue.location.line, issue.kind.display_name(), issue.kind.code())
        {
            issue.suppressed = true;
        }
    }
}

/// Append `UnusedSuppress` issues for any named `@suppress`/`@psalm-suppress`
/// annotations that did not match any issue in `all_issues`. The new issues are
/// themselves subject to suppression (so `@suppress UnusedSuppress` works).
fn emit_unused_suppressions(
    all_issues: &mut Vec<Issue>,
    suppressions: &crate::suppression::SuppressionMap,
    file: &std::sync::Arc<str>,
) {
    let all_refs: Vec<&Issue> = all_issues.iter().collect();
    let pre_suppressed: Vec<&Issue> = all_refs.iter().filter(|i| i.suppressed).copied().collect();
    let unused = suppressions.unused_named(&all_refs, &pre_suppressed);
    for (line, kind) in unused {
        let loc = mir_types::Location::new(file.clone(), line, line, 0, 0);
        let mut issue = Issue::new(mir_issues::IssueKind::UnusedSuppress { kind }, loc);
        if suppressions.is_suppressed(line, issue.kind.display_name(), issue.kind.code()) {
            issue.suppressed = true;
        }
        all_issues.push(issue);
    }
}

/// Discover all `.php` files under a directory, recursively.
pub fn discover_files(root: &Path) -> Vec<PathBuf> {
    if root.is_file() {
        return vec![root.to_path_buf()];
    }
    let mut files = Vec::new();
    collect_php_files(root, &mut files);
    files
}

pub(crate) fn collect_php_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_symlink()).unwrap_or(false) {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if matches!(
                    name,
                    "vendor" | ".git" | "node_modules" | ".cache" | ".pnpm-store"
                ) {
                    continue;
                }
                collect_php_files(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("php") {
                out.push(path);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// FQCN reference walk — collects every class-name reference reachable from a
// ClassLike's signature surface. Used by lazy_load_missing_classes to chase
// transitive vendor types.
// ---------------------------------------------------------------------------

pub(crate) fn collect_class_referenced_fqcns(class: &crate::db::ClassLike, out: &mut Vec<String>) {
    if let Some(p) = class.parent() {
        out.push(p.to_string());
    }
    for i in class.interfaces() {
        out.push(i.to_string());
    }
    for e in class.extends() {
        out.push(e.to_string());
    }
    for t in class.class_traits() {
        out.push(t.to_string());
    }
    for m in class.mixins() {
        out.push(m.to_string());
    }
    for u in class.extends_type_args() {
        collect_fqcns_in_union(u, out);
    }
    for (iface, args) in class.implements_type_args() {
        out.push(iface.to_string());
        for u in args {
            collect_fqcns_in_union(u, out);
        }
    }
    for (_, m) in class.own_methods().iter() {
        for p in m.params.iter() {
            if let Some(t) = &p.ty {
                collect_fqcns_in_union(t, out);
            }
        }
        if let Some(t) = &m.return_type {
            collect_fqcns_in_union(t, out);
        }
        for thrown in m.throws.iter() {
            out.push(thrown.to_string());
        }
    }
    if let Some(props) = class.own_properties() {
        for (_, p) in props.iter() {
            if let Some(t) = &p.ty {
                collect_fqcns_in_union(t, out);
            }
        }
    }
    for (_, c) in class.own_constants().iter() {
        collect_fqcns_in_union(&c.ty, out);
    }
}

pub(crate) fn collect_fqcns_in_union(u: &Type, out: &mut Vec<String>) {
    for atom in u.types.iter() {
        collect_fqcns_in_atomic(atom, out);
    }
}

fn collect_fqcns_in_simple(t: &mir_types::compact::SimpleType, out: &mut Vec<String>) {
    if let mir_types::compact::SimpleType::Complex(u) = t {
        collect_fqcns_in_union(u, out);
    }
}

pub(crate) fn collect_fqcns_in_atomic(a: &Atomic, out: &mut Vec<String>) {
    match a {
        Atomic::TNamedObject { fqcn, type_params } => {
            out.push(fqcn.to_string());
            for tp in type_params.iter() {
                collect_fqcns_in_union(tp, out);
            }
        }
        Atomic::TStaticObject { fqcn } | Atomic::TSelf { fqcn } | Atomic::TParent { fqcn } => {
            out.push(fqcn.to_string());
        }
        Atomic::TLiteralEnumCase { enum_fqcn, .. } => {
            out.push(enum_fqcn.to_string());
        }
        Atomic::TClassString(Some(s)) | Atomic::TInterfaceString(Some(s)) => {
            out.push(s.to_string());
        }
        Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
            collect_fqcns_in_union(key, out);
            collect_fqcns_in_union(value, out);
        }
        Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
            collect_fqcns_in_union(value, out);
        }
        Atomic::TKeyedArray { properties, .. } => {
            for (_, kp) in properties.iter() {
                collect_fqcns_in_union(&kp.ty, out);
            }
        }
        Atomic::TClosure { data } => {
            for p in data.params.iter() {
                if let Some(t) = &p.ty {
                    collect_fqcns_in_simple(t, out);
                }
            }
            collect_fqcns_in_union(&data.return_type, out);
            if let Some(t) = &data.this_type {
                collect_fqcns_in_union(t, out);
            }
        }
        Atomic::TCallable {
            params,
            return_type,
        } => {
            if let Some(ps) = params {
                for p in ps {
                    if let Some(t) = &p.ty {
                        collect_fqcns_in_simple(t, out);
                    }
                }
            }
            if let Some(rt) = return_type {
                collect_fqcns_in_union(rt, out);
            }
        }
        Atomic::TIntersection { parts } => {
            for p in parts.iter() {
                collect_fqcns_in_union(p, out);
            }
        }
        Atomic::TConditional { data } => {
            collect_fqcns_in_union(&data.subject, out);
            collect_fqcns_in_union(&data.if_true, out);
            collect_fqcns_in_union(&data.if_false, out);
        }
        Atomic::TTemplateParam { as_type, .. } => {
            collect_fqcns_in_union(as_type, out);
        }
        _ => {}
    }
}

fn build_reverse_deps(db: &dyn crate::db::MirDatabase) -> HashMap<String, HashSet<String>> {
    let mut reverse: HashMap<String, HashSet<String>> = HashMap::default();

    let mut add_edge = |symbol: &str, dependent_file: &str| {
        if let Some(defining_file) = db.symbol_defining_file(symbol) {
            let def = defining_file.as_ref().to_string();
            if def != dependent_file {
                reverse
                    .entry(def)
                    .or_default()
                    .insert(dependent_file.to_string());
            }
        }
    };

    for (file, imports) in db.file_import_snapshots() {
        let file = file.as_ref().to_string();
        for fqcn in imports.values() {
            add_edge(fqcn.as_str(), &file);
        }
    }

    let extract_named_objects = |union: &mir_types::Type| {
        union
            .types
            .iter()
            .filter_map(|atomic| match atomic {
                mir_types::atomic::Atomic::TNamedObject { fqcn, .. } => Some(*fqcn),
                _ => None,
            })
            .collect::<Vec<_>>()
    };

    for fqcn in crate::db::workspace_classes(db).iter() {
        let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
        let Some(class) = crate::db::find_class_like(db, here) else {
            continue;
        };
        if class.is_interface() || class.is_trait() || class.is_enum() {
            continue;
        }
        let Some(file) = db
            .symbol_defining_file(fqcn.as_ref())
            .map(|f| f.as_ref().to_string())
            .or_else(|| class.location().map(|l| l.file.as_ref().to_string()))
        else {
            continue;
        };

        if let Some(parent) = class.parent() {
            add_edge(parent.as_ref(), &file);
        }
        for iface in class.interfaces().iter() {
            add_edge(iface.as_ref(), &file);
        }
        for tr in class.class_traits().iter() {
            add_edge(tr.as_ref(), &file);
        }
        if let Some(props) = class.own_properties() {
            for (_, p) in props.iter() {
                if let Some(ty) = &p.ty {
                    for named in extract_named_objects(ty) {
                        add_edge(named.as_ref(), &file);
                    }
                }
            }
        }
        for (_, method) in class.own_methods().iter() {
            for param in method.params.iter() {
                if let Some(ty) = &param.ty {
                    for named in extract_named_objects(ty.as_ref()) {
                        add_edge(named.as_ref(), &file);
                    }
                }
            }
            if let Some(rt) = method.return_type.as_deref() {
                for named in extract_named_objects(rt) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }
    }

    for fqn in crate::db::workspace_functions(db).iter() {
        let here = crate::db::Fqcn::from_str(db, fqn.as_ref());
        let Some(f) = crate::db::find_function(db, here) else {
            continue;
        };
        let Some(file) = db
            .symbol_defining_file(fqn.as_ref())
            .map(|f| f.as_ref().to_string())
            .or_else(|| f.location.as_ref().map(|l| l.file.as_ref().to_string()))
        else {
            continue;
        };

        for param in f.params.iter() {
            if let Some(ty) = &param.ty {
                for named in extract_named_objects(ty.as_ref()) {
                    add_edge(named.as_ref(), &file);
                }
            }
        }
        if let Some(rt) = f.return_type.as_deref() {
            for named in extract_named_objects(rt) {
                add_edge(named.as_ref(), &file);
            }
        }
    }

    for (ref_file, symbol_key) in db.all_reference_location_pairs() {
        let file_str = ref_file.as_ref().to_string();
        let lookup = crate::defining_file_lookup_key(&symbol_key);
        add_edge(lookup, &file_str);
    }

    reverse
}

fn extract_reference_locations(
    db: &dyn crate::db::MirDatabase,
    file: &Arc<str>,
) -> Arc<[crate::cache::CachedRefLoc]> {
    db.extract_file_reference_locations(file.as_ref()).into()
}

pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    #[doc(hidden)]
    pub type_envs: rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
    /// Per-expression resolved symbols from body analysis, sorted by file path.
    pub symbols: Vec<crate::symbol::ResolvedSymbol>,
    /// Maps each file path to the contiguous range within `symbols` that
    /// belongs to it.
    symbols_by_file: HashMap<Arc<str>, std::ops::Range<usize>>,
}

impl AnalysisResult {
    fn build(
        issues: Vec<Issue>,
        type_envs: rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        mut symbols: Vec<crate::symbol::ResolvedSymbol>,
    ) -> Self {
        symbols.sort_unstable_by(|a, b| a.file.as_ref().cmp(b.file.as_ref()));
        let mut symbols_by_file: HashMap<Arc<str>, std::ops::Range<usize>> = HashMap::default();
        let mut i = 0;
        while i < symbols.len() {
            let file = Arc::clone(&symbols[i].file);
            let start = i;
            while i < symbols.len() && symbols[i].file == file {
                i += 1;
            }
            symbols_by_file.insert(file, start..i);
        }
        Self {
            issues,
            type_envs,
            symbols,
            symbols_by_file,
        }
    }

    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == mir_issues::Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == mir_issues::Severity::Warning)
            .count()
    }

    pub fn issues_by_file(&self) -> HashMap<Arc<str>, Vec<&Issue>> {
        let mut map: HashMap<Arc<str>, Vec<&Issue>> = HashMap::default();
        for issue in &self.issues {
            map.entry(issue.location.file.clone())
                .or_default()
                .push(issue);
        }
        map
    }

    pub fn count_by_severity(&self) -> Vec<(mir_issues::Severity, usize)> {
        let mut counts: std::collections::BTreeMap<mir_issues::Severity, usize> =
            std::collections::BTreeMap::new();
        for issue in &self.issues {
            *counts.entry(issue.severity).or_insert(0) += 1;
        }
        counts.into_iter().collect()
    }

    pub fn total_issue_count(&self) -> usize {
        self.issues.len()
    }

    pub fn filter_issues<'a, F>(&'a self, predicate: F) -> impl Iterator<Item = &'a Issue>
    where
        F: Fn(&Issue) -> bool + 'a,
    {
        self.issues.iter().filter(move |i| predicate(i))
    }

    pub fn symbol_at(
        &self,
        file: &str,
        byte_offset: u32,
    ) -> Option<&crate::symbol::ResolvedSymbol> {
        let range = self.symbols_by_file.get(file)?;
        let symbols = &self.symbols[range.clone()];

        // Primary: cursor is on an identifier token.
        if let Some(sym) = symbols
            .iter()
            .filter(|s| s.span.start <= byte_offset && byte_offset < s.span.end)
            .min_by_key(|s| s.span.end - s.span.start)
        {
            return Some(sym);
        }

        // Fallback: cursor is in a call-expression gap (e.g. the whitespace or
        // argument list between two chained method calls).  Match against the
        // full expression span recorded for call-like symbols and return the
        // innermost (smallest) enclosing call, mirroring what an AST-walk to
        // the innermost containing call expression would produce.
        symbols
            .iter()
            .filter(|s| {
                s.expr_span
                    .is_some_and(|es| es.start <= byte_offset && byte_offset < es.end)
            })
            .min_by_key(|s| {
                let es = s.expr_span.unwrap();
                es.end - es.start
            })
    }
}
