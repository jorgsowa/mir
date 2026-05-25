//! Per-expression resolved symbol data, retained from body analysis.
//!
//! The static analyzer already resolves types for every expression during
//! analysis but historically discarded the intermediate state.  This module
//! exposes that data so that downstream tools can build position indexes for
//! hover, go-to-definition, and completions.

use std::sync::Arc;

use mir_types::Union;
use php_ast::Span;

/// A single resolved symbol observed during body analysis.
#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
    /// Absolute path of the file this symbol was found in.
    pub file: Arc<str>,
    /// Byte-offset span in the source file.
    pub span: Span,
    /// What kind of symbol this is.
    pub kind: SymbolKind,
    /// The resolved type at this location.
    pub resolved_type: Union,
}

impl ResolvedSymbol {
    /// Return the key used in the salsa db's reference-location table for this
    /// symbol, or `None` for kinds that are not tracked there (e.g. variables).
    ///
    /// Key format mirrors `MirDatabase::record_reference_location`:
    /// - method / static call : `"ClassName::methodname"` (method lowercased)
    /// - property access      : `"ClassName::propName"`
    /// - function call        : fully-qualified function name
    /// - class reference      : fully-qualified class name
    ///
    /// Prefer [`Self::to_symbol`] for type-safe access.
    pub fn codebase_key(&self) -> Option<String> {
        match &self.kind {
            SymbolKind::MethodCall { class, method } | SymbolKind::StaticCall { class, method } => {
                Some(format!("{}::{}", class, method.to_lowercase()))
            }
            SymbolKind::PropertyAccess { class, property } => Some(format!("{class}::{property}")),
            SymbolKind::FunctionCall(fqn) => Some(fqn.to_string()),
            SymbolKind::ClassReference(fqcn) => Some(fqcn.to_string()),
            SymbolKind::Variable(_) => None,
        }
    }

    /// Convert this `ResolvedSymbol` to a typed [`crate::Symbol`] for use with
    /// [`crate::AnalysisSession::definition_of`], [`crate::AnalysisSession::references_to`],
    /// or [`crate::AnalysisSession::hover`].
    ///
    /// Returns `None` for kinds that don't map to a codebase-level symbol
    /// (currently only `Variable` — local variables aren't tracked in the
    /// codebase symbol table).
    pub fn to_symbol(&self) -> Option<crate::Symbol> {
        match &self.kind {
            SymbolKind::MethodCall { class, method } | SymbolKind::StaticCall { class, method } => {
                Some(crate::Symbol::method(class.clone(), method.as_ref()))
            }
            SymbolKind::PropertyAccess { class, property } => {
                Some(crate::Symbol::property(class.clone(), property.clone()))
            }
            SymbolKind::FunctionCall(fqn) => Some(crate::Symbol::function(fqn.clone())),
            SymbolKind::ClassReference(fqcn) => Some(crate::Symbol::class(fqcn.clone())),
            SymbolKind::Variable(_) => None,
        }
    }
}

/// One declaration emitted by [`crate::AnalysisSession::document_symbols`].
/// Tool-agnostic shape for outline / breadcrumb features; consumers map this
/// onto whatever protocol-specific symbol type they need.
///
/// Forms a tree: classes/interfaces/traits/enums have `children` populated
/// with their methods, properties, and constants. Top-level functions and
/// constants have empty `children`.
#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    /// FQCN for classes/interfaces/traits/enums; FQN for functions /
    /// constants; short name for members nested inside a class.
    pub name: Arc<str>,
    /// Coarse kind suitable for icon / severity selection in outlines.
    pub kind: DocumentSymbolKind,
    /// Source location of the declaration (file + 1-based lines, 0-based
    /// columns). `None` only for synthetic stub-only definitions that don't
    /// have a recorded source span.
    pub location: Option<mir_codebase::storage::Location>,
    /// For container symbols (Class, Interface, Trait, Enum), the nested
    /// methods, properties, and constants declared on this type. Empty for
    /// leaf kinds (Function, Constant, etc.).
    pub children: Vec<DocumentSymbol>,
}

/// Coarse declaration kind used by [`DocumentSymbol`]. Maps onto outline-style
/// symbol kinds in any consumer protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSymbolKind {
    Class,
    Interface,
    Trait,
    Enum,
    Function,
    Method,
    Property,
    Constant,
    EnumCase,
}

/// The kind of symbol that was resolved.
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// A variable reference (`$foo`).
    Variable(Arc<str>),
    /// An instance method call (`$obj->method()`).
    MethodCall { class: Arc<str>, method: Arc<str> },
    /// A static method call (`Foo::method()`).
    StaticCall { class: Arc<str>, method: Arc<str> },
    /// A property access (`$obj->prop`).
    PropertyAccess { class: Arc<str>, property: Arc<str> },
    /// A free function call (`foo()`).
    FunctionCall(Arc<str>),
    /// A class reference (`new Foo`, `instanceof Foo`, type hints).
    ClassReference(Arc<str>),
}
