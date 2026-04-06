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

pub use project::{AnalysisResult, ProjectAnalyzer};
pub use stubs::is_builtin_function;

pub mod type_env;
pub use mir_issues::{Issue, IssueKind, Location, Severity};
pub use type_env::{ScopeId, TypeEnv};
