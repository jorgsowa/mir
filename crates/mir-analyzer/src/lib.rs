pub(crate) mod arena;
#[doc(hidden)]
pub mod cache;
pub(crate) mod call;
pub(crate) mod class;
pub(crate) mod collector;
pub(crate) mod context;
#[doc(hidden)]
pub mod db;
pub(crate) mod dead_code;
pub(crate) mod diagnostics;
pub(crate) mod expr;
pub mod file_analyzer;
pub(crate) mod generic;
#[doc(hidden)]
pub mod metrics;
pub(crate) mod narrowing;
#[doc(hidden)]
pub mod parser;
pub(crate) mod pass2;
pub mod php_version;
pub mod project;
pub mod session;
pub(crate) mod shared_db;
pub mod source_provider;
pub(crate) mod stmt;
#[doc(hidden)]
pub mod stub_cache;
#[doc(hidden)]
pub mod stubs;
pub(crate) mod taint;
pub(crate) mod type_env;

pub use file_analyzer::{BatchFileAnalyzer, FileAnalysis, FileAnalyzer, ParsedFile};
pub use parser::type_from_hint::type_from_hint;
pub use parser::{DocblockParser, ParsedDocblock};
pub use php_version::{ParsePhpVersionError, PhpVersion};
pub use project::{AnalysisResult, ProjectAnalyzer};
pub use session::AnalysisSession;
pub use source_provider::{FsSourceProvider, SourceProvider};
pub use stubs::{
    is_builtin_function, stub_files, stub_path_for_class, ChainedClassResolver, StubClassResolver,
    StubVfs,
};

// ============================================================================
// API Unification: ProjectAnalyzer and AnalysisSession
// ============================================================================
//
// `ProjectAnalyzer` (batch-oriented) and `AnalysisSession` (incremental) are
// now unified under a single analysis engine. Both share the same Salsa database,
// definition collection, and Pass 2 type inference logic. The difference is
// ownership model and parallelization strategy:
//
// - `ProjectAnalyzer`: Owns the database and all files; analyzes them in parallel.
//   Best for CLI, CI, and bulk analysis. Configuration via public fields before
//   calling `analyze()`.
//
// - `AnalysisSession`: Incremental file-by-file analysis; clients ingest files
//   as they change. Best for LSP servers and watch modes. Configuration via
//   builder pattern (with_cache, with_psr4, etc.).
//
// New code should prefer `AnalysisSession` for flexibility; `ProjectAnalyzer`
// is maintained for backward compatibility with batch workflows.

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
pub enum Symbol {
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

impl Symbol {
    /// Construct a method symbol. Normalizes `name` to lowercase since PHP
    /// methods are case-insensitive.
    pub fn method(class: impl Into<std::sync::Arc<str>>, name: &str) -> Self {
        Symbol::Method {
            class: class.into(),
            name: std::sync::Arc::from(name.to_ascii_lowercase()),
        }
    }

    /// Construct a class symbol.
    pub fn class(fqcn: impl Into<std::sync::Arc<str>>) -> Self {
        Symbol::Class(fqcn.into())
    }

    /// Construct a function symbol.
    pub fn function(fqn: impl Into<std::sync::Arc<str>>) -> Self {
        Symbol::Function(fqn.into())
    }

    /// Construct a property symbol.
    pub fn property(
        class: impl Into<std::sync::Arc<str>>,
        name: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Symbol::Property {
            class: class.into(),
            name: name.into(),
        }
    }

    /// Construct a class constant symbol.
    pub fn class_constant(
        class: impl Into<std::sync::Arc<str>>,
        name: impl Into<std::sync::Arc<str>>,
    ) -> Self {
        Symbol::ClassConstant {
            class: class.into(),
            name: name.into(),
        }
    }

    /// Construct a global constant symbol.
    pub fn global_constant(fqn: impl Into<std::sync::Arc<str>>) -> Self {
        Symbol::GlobalConstant(fqn.into())
    }

    /// The codebase lookup key for this symbol (used internally for the
    /// reference-locations index). Stable across releases.
    pub fn codebase_key(&self) -> String {
        match self {
            Symbol::Class(fqcn) => fqcn.to_string(),
            Symbol::Function(fqn) => fqn.to_string(),
            Symbol::Method { class, name } => format!("{class}::{name}"),
            Symbol::Property { class, name } => format!("{class}::{name}"),
            Symbol::ClassConstant { class, name } => format!("{class}::{name}"),
            Symbol::GlobalConstant(fqn) => fqn.to_string(),
        }
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

/// Result of a lazy-load attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LazyLoadOutcome {
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
    pub definition: Option<mir_codebase::storage::Location>,
}

/// File dependency graph: tracks which files depend on which other files.
/// Used for incremental invalidation in LSP servers and build systems.
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Direct dependencies: file → [files it depends on]
    dependencies: std::collections::HashMap<String, Vec<String>>,
    /// Reverse dependencies: file → [files that depend on it]
    dependents: std::collections::HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Files that `file` directly depends on (imports, parent classes, interfaces, traits).
    pub fn dependencies_of(&self, file: &str) -> &[String] {
        self.dependencies
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Files that directly depend on `file` (reverse edge).
    pub fn dependents_of(&self, file: &str) -> &[String] {
        self.dependents
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// All files transitively depended upon by `file` (including indirect).
    pub fn transitive_dependencies(&self, file: &str) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![file.to_string()];
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for dep in self.dependencies_of(&current) {
                if !visited.contains(dep) {
                    queue.push(dep.clone());
                    result.push(dep.clone());
                }
            }
        }
        result
    }

    /// All files that transitively depend on `file` (reverse transitive).
    pub fn transitive_dependents(&self, file: &str) -> Vec<String> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = vec![file.to_string()];
        let mut result = Vec::new();

        while let Some(current) = queue.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            for dep in self.dependents_of(&current) {
                if !visited.contains(dep) {
                    queue.push(dep.clone());
                    result.push(dep.clone());
                }
            }
        }
        result
    }
}

pub mod symbol;
pub use mir_codebase::storage::{FnParam, TemplateParam, Visibility};
pub use mir_issues::{Issue, IssueKind, Location, Severity};
pub use mir_types::Union as Type;

/// Convert a parser [`php_ast::Span`] (byte-offset range) into a
/// [`mir_codebase::storage::Location`] (file path + 1-based line range +
/// 0-based codepoint columns) using `source` and the parser's `source_map`.
///
/// This is the canonical way for consumers to translate Pass-2 result spans
/// (e.g. [`crate::symbol::ResolvedSymbol::span`]) into source locations they
/// can hand to their own protocol layer. Consumers that need different
/// position semantics (LSP UTF-16 code units, byte offsets, etc.) translate
/// from this `Location` rather than re-implementing the column math.
pub fn location_from_span(
    span: php_ast::Span,
    file: std::sync::Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) -> mir_codebase::storage::Location {
    let (line, col_start) = diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = if span.start < span.end {
        diagnostics::offset_to_line_col(source, span.end, source_map)
    } else {
        (line, col_start)
    };
    mir_codebase::storage::Location {
        file,
        line,
        line_end,
        col_start,
        col_end: col_end.max(col_start.saturating_add(1)),
    }
}
pub use symbol::{DocumentSymbol, DocumentSymbolKind, ResolvedSymbol, SymbolKind};

pub mod composer;
pub use composer::{ComposerError, Psr4Map};
pub use type_env::ScopeId;

#[doc(hidden)]
pub mod test_utils;
