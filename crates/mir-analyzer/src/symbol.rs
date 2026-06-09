//! Per-expression resolved symbol data, retained from body analysis.
//!
//! The static analyzer already resolves types for every expression during
//! analysis but historically discarded the intermediate state.  This module
//! exposes that data so that downstream tools can build position indexes for
//! hover, go-to-definition, and completions.

use std::sync::Arc;

use mir_types::Type;
use php_ast::Span;

/// A single resolved symbol observed during body analysis.
#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
    /// Absolute path of the file this symbol was found in.
    pub file: Arc<str>,
    /// Byte-offset span covering only the identifier token (method name,
    /// function name, variable sigil+name, etc.).  Used for precise
    /// go-to-definition and reference highlighting.
    pub span: Span,
    /// Byte-offset span of the full call expression, e.g. the entire
    /// `$obj->method(args)` node.  Set only for call-like symbols (method
    /// calls, static calls, function calls).  `symbol_at` uses this as a
    /// fallback so that a cursor sitting in the whitespace between two
    /// chained method calls still resolves to the innermost enclosing call.
    pub expr_span: Option<Span>,
    /// What kind of symbol this is.
    pub kind: ReferenceKind,
    /// The resolved type at this location.
    pub resolved_type: Type,
}

impl ResolvedSymbol {
    /// Return the reference-index lookup key, or `None` for kinds that are not
    /// tracked there (e.g. variables). Delegates to [`ReferenceKind::to_name`].
    pub fn codebase_key(&self) -> Option<String> {
        self.kind.to_name().map(|name| name.codebase_key())
    }

    /// Convert to a typed [`crate::Name`] for use with
    /// [`crate::AnalysisSession::definition_of`], [`crate::AnalysisSession::references_to`],
    /// or [`crate::AnalysisSession::hover`]. Delegates to [`ReferenceKind::to_name`].
    pub fn to_symbol(&self) -> Option<crate::Name> {
        self.kind.to_name()
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
    pub kind: DeclarationKind,
    /// Source location of the declaration (file + 1-based lines, 0-based
    /// columns). `None` only for synthetic stub-only definitions that don't
    /// have a recorded source span.
    pub location: Option<mir_types::Location>,
    /// For container symbols (Class, Interface, Trait, Enum), the nested
    /// methods, properties, and constants declared on this type. Empty for
    /// leaf kinds (Function, Constant, etc.).
    pub children: Vec<DocumentSymbol>,
}

/// Coarse declaration kind used by [`DocumentSymbol`]. Maps onto outline-style
/// symbol kinds in any consumer protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationKind {
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

/// The kind of symbol reference that was resolved.
#[derive(Debug, Clone)]
pub enum ReferenceKind {
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
    /// A class constant access (`Config::VERSION`, `self::CONST`, `parent::CONST`).
    ConstantAccess { class: Arc<str>, constant: Arc<str> },
}

impl ReferenceKind {
    /// Map to a typed [`crate::Name`], or `None` for kinds that don't correspond
    /// to a codebase-level symbol (currently only `Variable`).
    pub fn to_name(&self) -> Option<crate::Name> {
        match self {
            ReferenceKind::MethodCall { class, method }
            | ReferenceKind::StaticCall { class, method } => {
                Some(crate::Name::method(class.clone(), method.as_ref()))
            }
            ReferenceKind::PropertyAccess { class, property } => {
                Some(crate::Name::property(class.clone(), property.clone()))
            }
            ReferenceKind::FunctionCall(fqn) => Some(crate::Name::function(fqn.clone())),
            ReferenceKind::ClassReference(fqcn) => Some(crate::Name::class(fqcn.clone())),
            ReferenceKind::ConstantAccess { class, constant } => {
                Some(crate::Name::class_constant(class.clone(), constant.clone()))
            }
            ReferenceKind::Variable(_) => None,
        }
    }
}
