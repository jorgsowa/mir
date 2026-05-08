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
pub use symbol::{DocumentSymbol, DocumentSymbolKind, ResolvedSymbol, SymbolKind};
pub use type_env::{ScopeId, TypeEnv};

pub mod composer;
pub use composer::Psr4Map;

pub mod test_utils;
