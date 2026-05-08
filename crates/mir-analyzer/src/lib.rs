pub mod cache;
pub mod call;
pub mod class;
pub mod collector;
pub mod context;
pub mod db;
pub mod dead_code;
pub mod diagnostics;
pub mod expr;
pub mod file_analyzer;
pub mod generic;
pub mod narrowing;
pub mod parser;
pub mod pass2;
pub mod php_version;
pub mod project;
pub mod session;
pub mod stmt;
pub mod stubs;
pub mod taint;

pub use file_analyzer::{FileAnalysis, FileAnalyzer};
pub use parser::type_from_hint::type_from_hint;
pub use parser::{DocblockParser, ParsedDocblock};
pub use php_version::{ParsePhpVersionError, PhpVersion};
pub use project::{AnalysisResult, ProjectAnalyzer};
pub use session::AnalysisSession;
pub use stubs::{is_builtin_function, stub_files, StubVfs};

pub mod symbol;
pub mod type_env;
pub use mir_issues::{Issue, IssueKind, Location, Severity};

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
pub use type_env::{ScopeId, TypeEnv};

pub mod composer;
pub use composer::Psr4Map;

pub mod test_utils;
