pub mod cache;
pub mod call;
pub mod class;
pub mod collector;
pub mod context;
pub mod dead_code;
pub mod expr;
pub mod generic;
pub mod narrowing;
pub mod parser;
pub mod project;
pub mod stmt;
pub mod stubs;
pub mod taint;

pub use parser::type_from_hint::type_from_hint;
pub use parser::{DocblockParser, ParsedDocblock};
pub use project::{AnalysisResult, ProjectAnalyzer};
pub use stubs::is_builtin_function;

pub mod symbol;
pub mod type_env;
pub use mir_issues::{Issue, IssueKind, Location, Severity};
pub use symbol::{ResolvedSymbol, SymbolKind};
pub use type_env::{ScopeId, TypeEnv};

pub mod composer;
pub use composer::Psr4Map;

pub mod test_utils;
