use rustc_hash::FxHashMap;
use std::sync::{Arc, OnceLock};

pub(crate) mod analyzer_db;
pub(crate) mod attributes;
pub mod batch;
pub(crate) mod body_analysis;
#[doc(hidden)]
pub mod cache;
pub(crate) mod call;
pub(crate) mod class;
pub(crate) mod collector;
pub(crate) mod contradiction;
#[doc(hidden)]
pub mod db;
pub(crate) mod dead_code;
pub(crate) mod diagnostics;
pub(crate) mod expr;
pub mod file_analyzer;
pub(crate) mod flow_state;
pub(crate) mod generic;
pub mod indexing;
#[doc(hidden)]
pub mod metrics;
pub(crate) mod narrowing;
#[doc(hidden)]
pub mod parse_cache;
#[doc(hidden)]
pub mod parser;
pub mod php_version;
pub mod prelude;
pub mod session;
pub mod source_provider;
pub(crate) mod stmt;
#[doc(hidden)]
pub mod stub_cache;
#[doc(hidden)]
pub mod stubs;
pub(crate) mod subtype;
pub mod suppression;
pub(crate) mod taint;
pub(crate) mod type_env;
pub(crate) mod util;

pub use batch::{
    analyze_source, dead_code_issue_kinds, discover_files, AnalysisResult, BatchOptions,
};
pub use file_analyzer::{FileAnalysis, FileAnalyzer};
pub use indexing::{IndexBatchOutcome, IndexCancel, IndexParallelism};
pub use parser::type_from_hint::type_from_hint;
pub use parser::{DocblockParser, ParsedDocblock};
pub use php_version::{ParsePhpVersionError, PhpVersion};
pub use session::{AnalysisSession, SubtypeClassSite};
pub use source_provider::{FsSourceProvider, SourceProvider};

/// Returns `Some((used, canonical))` when `written` and `canonical` FQCNs differ only in casing.
/// Uses the short (last-segment) form when only the final segment is wrong and the namespace
/// prefix is already correct; otherwise returns the full path so the mismatch is visible.
pub(crate) fn fqcn_case_mismatch(written: &str, canonical: &str) -> Option<(String, String)> {
    let w = written.trim_start_matches('\\');
    let c = canonical.trim_start_matches('\\');
    if w == c || !w.eq_ignore_ascii_case(c) {
        return None;
    }
    let w_last = w.rsplit('\\').next().unwrap_or(w);
    let c_last = c.rsplit('\\').next().unwrap_or(c);
    if w_last != c_last {
        let w_prefix = w.rsplit_once('\\').map_or("", |(p, _)| p);
        let c_prefix = c.rsplit_once('\\').map_or("", |(p, _)| p);
        if w_prefix == c_prefix {
            return Some((w_last.to_string(), c_last.to_string()));
        }
    }
    Some((w.to_string(), c.to_string()))
}
pub use stubs::{
    is_builtin_constant, is_builtin_function, stub_files, stub_path_for_class,
    ChainedClassResolver, StubClassResolver, StubVfs,
};

// ============================================================================
// Analysis entry points
// ============================================================================
//
// `AnalysisSession` is the single analysis engine. It supports two usage modes:
//
// - Batch (CLI, CI, bulk analysis): use `analyze_paths` / `BatchOptions` to
//   run definition collection and body analysis over many files in parallel.
//
// - Incremental (LSP, watch mode): ingest files as they change; per-file
//   results come from `FileAnalyzer::analyze`. Builder-style configuration
//   (`with_cache`, `with_psr4`, …).
//
// The two phases of analysis are:
//   1. Definition collection — discovers classes, functions, constants in a
//      file and registers them in the salsa database.
//   2. Body analysis (`BodyAnalyzer`) — walks function/method bodies,
//      inferring types and emitting issues.

/// A position in source code: 1-based line, 0-based codepoint column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub line: u32,
    pub column: u32,
}

/// A range in source code: start and end positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// A semantic identifier for a code entity that the analyzer can resolve.
///
/// Replaces the previous stringly-typed `&str` keys. Method names are
/// normalized (lowercased) at construction since PHP method dispatch is
/// case-insensitive — this prevents a class of correctness bugs where
/// callers pass mixed-case names and get empty results.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Name {
    /// A class, interface, trait, or enum (FQCN).
    Class(std::sync::Arc<str>),
    /// A global function (FQN).
    Function(std::sync::Arc<str>),
    /// An instance or static method.
    Method {
        class: std::sync::Arc<str>,
        name: std::sync::Arc<str>,
    },
    /// A class property.
    Property {
        class: std::sync::Arc<str>,
        name: std::sync::Arc<str>,
    },
    /// A class / interface / enum constant.
    ClassConstant {
        class: std::sync::Arc<str>,
        name: std::sync::Arc<str>,
    },
    /// A global constant.
    GlobalConstant(std::sync::Arc<str>),
}

impl Name {
    /// Construct a method symbol. Normalizes `name` to lowercase since PHP
    /// methods are case-insensitive.
    pub fn method(class: impl Into<std::sync::Arc<str>>, name: &str) -> Self {
        Name::Method {
            class: class.into(),
            name: std::sync::Arc::from(name.to_ascii_lowercase()),
        }
    }

    /// Construct a class symbol.
    pub fn class(fqcn: impl Into<std::sync::Arc<str>>) -> Self {
        Name::Class(fqcn.into())
    }

    /// Construct a function symbol.
    pub fn function(fqn: impl Into<std::sync::Arc<str>>) -> Self {
        Name::Function(fqn.into())
    }

    /// Construct a property symbol.
    pub fn property(
        class: impl Into<std::sync::Arc<str>>,
        name: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Name::Property {
            class: class.into(),
            name: name.into(),
        }
    }

    /// Construct a class constant symbol.
    pub fn class_constant(
        class: impl Into<std::sync::Arc<str>>,
        name: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Name::ClassConstant {
            class: class.into(),
            name: name.into(),
        }
    }

    /// Construct a global constant symbol.
    pub fn global_constant(fqn: impl Into<std::sync::Arc<str>>) -> Self {
        Name::GlobalConstant(fqn.into())
    }

    /// The codebase lookup key for this symbol (used internally for the
    /// reference-locations index). Stable across releases.
    ///
    /// Kind-prefixed (`cls:`, `fn:`, `meth:`, `prop:`, `cnst:`, `gcnst:`) so
    /// that a method, property, and class constant sharing the same class and
    /// name (e.g. `Foo::bar` as both a property and a method) never collide
    /// on the same reference-index entry — an unprefixed scheme merged their
    /// usages, which both scrambled `references_to` results and hid truly
    /// dead members behind an unrelated same-named symbol's usage.
    pub fn codebase_key(&self) -> String {
        match self {
            Name::Class(fqcn) => format!("cls:{fqcn}"),
            Name::Function(fqn) => format!("fn:{fqn}"),
            Name::Method { class, name } => format!("meth:{class}::{name}"),
            Name::Property { class, name } => format!("prop:{class}::{name}"),
            Name::ClassConstant { class, name } => format!("cnst:{class}::{name}"),
            Name::GlobalConstant(fqn) => format!("gcnst:{fqn}"),
        }
    }
}

/// Reduce a reference-index symbol key (as produced by [`Name::codebase_key`])
/// to the bare class/function/global-constant name that
/// `MirDatabase::symbol_defining_file` is keyed by.
///
/// Strips the kind prefix, then — for member keys (`meth:`/`prop:`/`cnst:`,
/// shaped `Class::member`) — keeps only the class portion, since a member has
/// no defining file of its own; only its owning class does.
pub(crate) fn defining_file_lookup_key(symbol_key: &str) -> &str {
    let stripped = symbol_key
        .strip_prefix("meth:")
        .or_else(|| symbol_key.strip_prefix("prop:"))
        .or_else(|| symbol_key.strip_prefix("cnst:"))
        .or_else(|| symbol_key.strip_prefix("cls:"))
        .or_else(|| symbol_key.strip_prefix("fn:"))
        .or_else(|| symbol_key.strip_prefix("gcnst:"))
        .unwrap_or(symbol_key);
    match stripped.split_once("::") {
        Some((class, _)) => class,
        None => stripped,
    }
}

/// Reason a symbol lookup did not return a location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolLookupError {
    /// No such symbol exists in the codebase.
    NotFound,
    /// The symbol exists but has no recorded source location (e.g. a
    /// stub-only declaration without a span).
    NoSourceLocation,
}

/// Outcome of a [`AnalysisSession::load_class`] attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOutcome {
    /// The symbol was already present in the session; no work performed.
    AlreadyLoaded,
    /// The symbol was resolved by the configured [`ClassResolver`] and the
    /// defining file was ingested.
    Loaded,
    /// No resolver is configured, the resolver could not map the FQCN to a
    /// file, or the resolved file could not be read / did not define the
    /// requested symbol.
    NotResolvable,
}

impl LoadOutcome {
    /// `true` when the symbol is now present in the session (whether it was
    /// already there or just freshly loaded).
    pub fn is_loaded(self) -> bool {
        !matches!(self, LoadOutcome::NotResolvable)
    }
}

/// Pluggable strategy for mapping a fully-qualified class name to the file
/// that should define it. The analyzer never touches `vendor/` or the
/// filesystem on its own — it asks a `ClassResolver` when a symbol is needed.
///
/// `mir_analyzer::Psr4Map` is the built-in implementation for Composer-based
/// projects. Consumers with non-Composer conventions (WordPress, Drupal, a
/// custom autoloader, a workspace-walk index) supply their own.
pub trait ClassResolver: Send + Sync {
    /// Resolve `fqcn` to the file that defines it. Returning `None` causes
    /// the analyzer to fall back to emitting `UndefinedClass`.
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf>;
}

impl ClassResolver for composer::Psr4Map {
    fn resolve(&self, fqcn: &str) -> Option<std::path::PathBuf> {
        composer::Psr4Map::resolve(self, fqcn)
    }
}

impl std::fmt::Display for SymbolLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolLookupError::NotFound => write!(f, "symbol not found"),
            SymbolLookupError::NoSourceLocation => write!(f, "symbol has no source location"),
        }
    }
}

impl std::error::Error for SymbolLookupError {}

/// Hover information for a symbol at a source location.
/// Includes the inferred type, optional docstring, and location of definition.
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Inferred type of the symbol.
    pub ty: Type,
    /// Docstring / documentation comment for the symbol (if available).
    pub docstring: Option<String>,
    /// Source location of the symbol's definition.
    pub definition: Option<mir_types::Location>,
}

/// File dependency graph: tracks which files depend on which other files.
/// Used for incremental invalidation in LSP servers and build systems.
#[derive(Debug)]
pub struct DependencyGraph {
    files: Vec<Arc<str>>,
    file_ids: FxHashMap<Arc<str>, u32>,
    /// Direct dependencies: file id → [file ids it depends on]
    dependencies: Vec<Vec<u32>>,
    /// Reverse dependencies: file id → [file ids that depend on it]
    dependents: Vec<Vec<u32>>,
    legacy_dependencies: OnceLock<FxHashMap<String, Vec<String>>>,
    legacy_dependents: OnceLock<FxHashMap<String, Vec<String>>>,
}

impl Clone for DependencyGraph {
    fn clone(&self) -> Self {
        Self {
            files: self.files.clone(),
            file_ids: self.file_ids.clone(),
            dependencies: self.dependencies.clone(),
            dependents: self.dependents.clone(),
            legacy_dependencies: OnceLock::new(),
            legacy_dependents: OnceLock::new(),
        }
    }
}

impl DependencyGraph {
    pub(crate) fn from_compact_parts(
        files: Vec<Arc<str>>,
        file_ids: FxHashMap<Arc<str>, u32>,
        dependencies: Vec<Vec<u32>>,
        dependents: Vec<Vec<u32>>,
    ) -> Self {
        Self {
            files,
            file_ids,
            dependencies,
            dependents,
            legacy_dependencies: OnceLock::new(),
            legacy_dependents: OnceLock::new(),
        }
    }

    /// Number of files tracked by this graph.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Number of direct dependency edges.
    pub fn dependency_edge_count(&self) -> usize {
        self.dependencies.iter().map(Vec::len).sum()
    }

    /// Number of reverse dependency edges.
    pub fn dependent_edge_count(&self) -> usize {
        self.dependents.iter().map(Vec::len).sum()
    }

    fn file_path(&self, id: u32) -> &Arc<str> {
        &self.files[id as usize]
    }

    fn stringify_adjacency(&self, adjacency: &[Vec<u32>]) -> FxHashMap<String, Vec<String>> {
        adjacency
            .iter()
            .enumerate()
            .filter(|(_, deps)| !deps.is_empty())
            .map(|(file_id, deps)| {
                (
                    self.files[file_id].to_string(),
                    deps.iter()
                        .map(|&dep_id| self.file_path(dep_id).to_string())
                        .collect(),
                )
            })
            .collect()
    }

    fn direct_arcs<'a>(
        &'a self,
        file: &str,
        adjacency: &'a [Vec<u32>],
    ) -> impl Iterator<Item = &'a Arc<str>> {
        self.file_ids
            .get(file)
            .and_then(|&id| adjacency.get(id as usize))
            .into_iter()
            .flatten()
            .map(|&id| self.file_path(id))
    }

    /// Files that `file` directly depends on, as interned paths.
    pub fn dependency_paths_of(&self, file: &str) -> Vec<Arc<str>> {
        self.direct_arcs(file, &self.dependencies)
            .cloned()
            .collect()
    }

    /// Files that directly depend on `file`, as interned paths.
    pub fn dependent_paths_of(&self, file: &str) -> Vec<Arc<str>> {
        self.direct_arcs(file, &self.dependents).cloned().collect()
    }

    /// Files that `file` directly depends on (imports, parent classes, interfaces, traits).
    pub fn dependencies_of(&self, file: &str) -> &[String] {
        self.legacy_dependencies
            .get_or_init(|| self.stringify_adjacency(&self.dependencies))
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Files that directly depend on `file` (reverse edge).
    pub fn dependents_of(&self, file: &str) -> &[String] {
        self.legacy_dependents
            .get_or_init(|| self.stringify_adjacency(&self.dependents))
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// All files transitively depended upon by `file` (including indirect).
    pub fn transitive_dependencies(&self, file: &str) -> Vec<String> {
        let mut visited = rustc_hash::FxHashSet::default();
        let Some(&seed) = self.file_ids.get(file) else {
            return Vec::new();
        };
        let mut queue = vec![seed];
        visited.insert(seed);
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            for &dep in &self.dependencies[current as usize] {
                if visited.insert(dep) {
                    queue.push(dep);
                    result.push(self.file_path(dep).to_string());
                }
            }
        }
        result
    }

    /// All files that transitively depend on `file` (reverse transitive).
    pub fn transitive_dependents(&self, file: &str) -> Vec<String> {
        let mut visited = rustc_hash::FxHashSet::default();
        let Some(&seed) = self.file_ids.get(file) else {
            return Vec::new();
        };
        let mut queue = vec![seed];
        visited.insert(seed);
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            for &dep in &self.dependents[current as usize] {
                if visited.insert(dep) {
                    queue.push(dep);
                    result.push(self.file_path(dep).to_string());
                }
            }
        }
        result
    }
}

pub mod symbol;
pub use mir_codebase::definitions::{DeclaredParam, FunctionDef, TemplateParam, Visibility};
pub use mir_issues::{Issue, IssueKind, Severity};
pub use mir_types::Type;

#[cfg(test)]
mod dependency_graph_tests {
    use super::*;

    fn graph(edges: &[(&str, &[&str])]) -> DependencyGraph {
        let mut files: Vec<Arc<str>> = edges
            .iter()
            .flat_map(|(from, deps)| {
                std::iter::once(*from)
                    .chain(deps.iter().copied())
                    .map(Arc::<str>::from)
            })
            .collect();
        files.sort();
        files.dedup();

        let file_ids: FxHashMap<Arc<str>, u32> = files
            .iter()
            .enumerate()
            .map(|(id, file)| (file.clone(), id as u32))
            .collect();
        let mut dependencies = vec![Vec::new(); files.len()];
        let mut dependents = vec![Vec::new(); files.len()];

        for (from, deps) in edges {
            let from_id = file_ids[*from];
            for dep in *deps {
                let dep_id = file_ids[*dep];
                dependencies[from_id as usize].push(dep_id);
                dependents[dep_id as usize].push(from_id);
            }
        }
        for adjacency in [&mut dependencies, &mut dependents] {
            for deps in adjacency {
                deps.sort();
                deps.dedup();
            }
        }

        DependencyGraph::from_compact_parts(files, file_ids, dependencies, dependents)
    }

    #[test]
    fn transitive_traversal_deduplicates_diamond_graphs() {
        let graph = graph(&[
            ("a.php", &["b.php", "c.php"]),
            ("b.php", &["d.php"]),
            ("c.php", &["d.php"]),
        ]);

        let deps = graph.transitive_dependencies("a.php");
        assert_eq!(
            deps.iter().filter(|path| path.as_str() == "d.php").count(),
            1
        );
        assert_eq!(deps.len(), 3);

        let dependents = graph.transitive_dependents("d.php");
        assert_eq!(
            dependents
                .iter()
                .filter(|path| path.as_str() == "a.php")
                .count(),
            1
        );
        assert_eq!(dependents.len(), 3);
    }

    #[test]
    fn direct_arc_accessors_do_not_require_legacy_string_maps() {
        let graph = graph(&[("consumer.php", &["service.php"])]);

        let deps = graph.dependency_paths_of("consumer.php");
        assert_eq!(deps, vec![Arc::<str>::from("service.php")]);
        assert!(graph.legacy_dependencies.get().is_none());

        let dependents = graph.dependent_paths_of("service.php");
        assert_eq!(dependents, vec![Arc::<str>::from("consumer.php")]);
        assert!(graph.legacy_dependents.get().is_none());

        assert_eq!(
            graph.dependencies_of("consumer.php"),
            &["service.php".to_string()]
        );
        assert!(graph.legacy_dependencies.get().is_some());
    }
}

/// Convert a parser [`php_ast::Span`] (byte-offset range) into a
/// [`mir_types::Location`] (file path + 1-based line range +
/// 0-based codepoint columns) using `source` and the parser's `source_map`.
///
/// This is the canonical way for consumers to translate body-analysis result spans
/// (e.g. [`crate::symbol::ResolvedSymbol::span`]) into source locations they
/// can hand to their own protocol layer. Consumers that need different
/// position semantics (LSP UTF-16 code units, byte offsets, etc.) translate
/// from this `Location` rather than re-implementing the column math.
pub fn location_from_span(
    span: php_ast::Span,
    file: std::sync::Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) -> mir_types::Location {
    let (line, col_start) = diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = if span.start < span.end {
        diagnostics::offset_to_line_col(source, span.end, source_map)
    } else {
        (line, col_start)
    };
    mir_types::Location {
        file,
        line,
        line_end,
        col_start,
        col_end: diagnostics::clamp_col_end(line, line_end, col_start, col_end),
    }
}
pub use symbol::{DeclarationKind, DocumentSymbol, ReferenceKind, ResolvedSymbol};

pub mod composer;
pub use composer::{ComposerError, Psr4Map};
pub use type_env::ScopeId;

#[doc(hidden)]
pub mod test_utils;
