//! Convenience re-exports for common mir-analyzer use cases.
//!
//! Consumers who want the full kit of types from mir-analyzer and its
//! dependencies import from the prelude:
//!
//! ```ignore
//! use mir_analyzer::prelude::*;
//! ```
//!
//! This pulls in types from all four context crates:
//! - `mir-types`: [`Type`], [`Name`] (the interned identifier)
//! - `mir-codebase`: [`DeclaredParam`], [`TemplateParam`], [`Visibility`]
//! - `mir-issues`: [`Issue`], [`IssueKind`], [`Severity`]
//! - `mir-analyzer` itself: [`AnalysisSession`], [`FileAnalyzer`], …

pub use crate::{
    AnalysisSession, DocumentSymbol, FileAnalysis, FileAnalyzer, HoverInfo, LoadOutcome, Name,
    Position, Range, ReferenceKind, ResolvedSymbol, SymbolLookupError,
};
pub use mir_codebase::definitions::{DeclaredParam, TemplateParam, Visibility};
pub use mir_issues::{Issue, IssueKind, Severity};
pub use mir_types::Type;
