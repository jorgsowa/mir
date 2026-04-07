//! Per-expression resolved symbol data, retained from Pass 2.
//!
//! The static analyzer already resolves types for every expression during
//! analysis but historically discarded the intermediate state.  This module
//! exposes that data so that downstream consumers (e.g. php-lsp) can build
//! position indexes for hover, go-to-definition, and completions.

use std::sync::Arc;

use mir_types::Union;
use php_ast::Span;

/// A single resolved symbol observed during Pass 2.
#[derive(Debug, Clone)]
pub struct ResolvedSymbol {
    /// Byte-offset span in the source file.
    pub span: Span,
    /// What kind of symbol this is.
    pub kind: SymbolKind,
    /// The resolved type at this location.
    pub resolved_type: Union,
}

/// The kind of symbol that was resolved.
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// A variable reference (`$foo`).
    Variable(String),
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
