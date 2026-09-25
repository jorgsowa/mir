//! Literal initializer typing and static-property write scanning.
//!
//! A class property declared with a bare `array` type (native `: array` hint
//! or `@var array` docblock) but initialized with a literal
//! (`private static $defaults = ['Name' => null, ...]`) would otherwise read
//! back as bare `array`: consumers such as `array_keys()` would type the keys
//! `int|string` and report spurious `PossiblyInvalidArgument` (MIR0105) on
//! call sites like `array_keys(self::$defaults)`.
//!
//! This module contributes two refinements, both applied at collection time so
//! every downstream consumer (docblock `@var` resolution, `effective_property_ty`,
//! demand-driven inference) sees them without further changes:
//!
//! 1. `literal_type` rebuilds the exact type of a *literal* initializer
//!    expression (scalars, `null`, array literals; everything else — calls,
//!    constant fetches, `new` — is `None` and contributes nothing).
//! 2. `refine_static_array_default` replaces a declared bare `array` with the
//!    literal's shape **only when** the property cannot have been written by
//!    other code in its own file.
//!
//! # Soundness gate
//!
//! PHP scopes `private static` to the declaring class, so the *file* is the
//! complete write scope for such a property. `scan_static_property_writes`
//! walks the file once and records every static-property assignment
//! (`C::$x`, `self::$x`, `static::$x`, `parent::$x`, compound operators,
//! `++/--`) whose target can be resolved to a class FQCN. Refinement is
//! withheld — conservatively — when any of these hold:
//!
//! * the property is not `private static` (protected/public statics can be
//!   written from other files, which a single-file scan cannot see);
//! * the file contains any static-property write whose target could not be
//!   resolved (dynamic class references, `parent::` outside the class body,
//!   or a single-segment name that might be a `use` alias);
//! * the initializer is not a literal the scanner can type (yields `mixed`);
//! * the declared type is not exactly bare `array` (a declared
//!   `array<string, mixed>` is a user invariant — refining it onto the
//!   literal would invent new behavior);
//! * the property itself appears in the file's write set.
//!
//! A bare unbraced `namespace F\Q;` applies to all *following sibling*
//! declarations (PHP semantics, mirrored by the parser's `NsKind::Unbraced`),
//! so the scanner tracks the namespace across sibling statements.
use std::collections::HashSet;
use std::ops::ControlFlow;

use mir_types::{Atomic, Type};
use php_ast::owned::visitor::{
    walk_owned_block, walk_owned_expr, walk_owned_program, walk_owned_stmt, OwnedVisitor,
};
use php_ast::owned::{ArrayElement, Expr, ExprKind, Name, NamespaceBody, Program, Stmt, StmtKind};

/// Every static-property write target found in one source file, keyed by
/// lowercased `(class FQCN, property name)`. `unresolved` is `true` when any
/// static-property write in the file could not be resolved to a class FQCN
/// (dynamic class, `parent::` outside the class body, or a single-segment
/// name that might be a `use` alias); it disables refinement for the whole
/// file rather than risk under-typing a refined property.
#[derive(Default)]
pub(super) struct StaticWrites {
    pub(super) writes: HashSet<(String, String)>,
    pub(super) unresolved: bool,
}

/// Walks the program once, recording static-property write targets and
/// tracking the current namespace / class static scope.
struct WriteScanner {
    /// Lowercased current namespace (without a leading backslash).
    ns: Option<String>,
    /// Lowercased FQCN of the enclosing class/trait/enum, or `None` outside
    /// any class-like declaration.
    class: Option<String>,
    /// Outer static scope, restored when the current class body ends (supports
    /// class-in-class nesting; a namespace block saves and restores this too).
    saved_class: Option<String>,
    out: StaticWrites,
}

/// Scans a parsed file for static-property write targets (see [`StaticWrites`]).
pub(super) fn scan_static_property_writes(program: &Program) -> StaticWrites {
    let mut scanner = WriteScanner {
        ns: None,
        class: None,
        saved_class: None,
        out: StaticWrites::default(),
    };
    let _ = walk_owned_program(&mut scanner, program);
    scanner.out
}

impl OwnedVisitor for WriteScanner {
    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Namespace(decl) => {
                let ns = decl.name.as_ref().map(lower_name);
                match &decl.body {
                    NamespaceBody::Braced(block) => {
                        let old_ns = std::mem::replace(&mut self.ns, ns);
                        let old_class = self.class.clone();
                        // A namespaced body does not inherit the surrounding
                        // static scope.
                        self.class = None;
                        let flow = walk_owned_block(self, block);
                        self.ns = old_ns;
                        self.class = old_class;
                        return flow;
                    }
                    // Unbraced `namespace X;` applies to all subsequent
                    // sibling statements; fall through to walk it (no body)
                    // with the updated namespace in place.
                    NamespaceBody::Simple => {
                        self.ns = ns;
                    }
                }
            }
            StmtKind::Class(decl) => {
                self.enter_static_scope(
                    decl.name
                        .as_ref()
                        .and_then(|n| n.as_deref())
                        .map(str::to_owned),
                );
                let flow = walk_owned_stmt(self, stmt);
                self.exit_static_scope();
                return flow;
            }
            StmtKind::Trait(decl) => {
                self.enter_static_scope(decl.name.as_deref().map(str::to_owned));
                let flow = walk_owned_stmt(self, stmt);
                self.exit_static_scope();
                return flow;
            }
            StmtKind::Enum(decl) => {
                self.enter_static_scope(decl.name.as_deref().map(str::to_owned));
                let flow = walk_owned_stmt(self, stmt);
                self.exit_static_scope();
                return flow;
            }
            _ => {}
        }
        walk_owned_stmt(self, stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let ExprKind::Assign(a) = &expr.kind {
            self.classify_assign_target(&a.target);
        }
        walk_owned_expr(self, expr)
    }
}

impl WriteScanner {
    fn enter_static_scope(&mut self, short_name: Option<String>) {
        self.saved_class = self.class.clone();
        self.class = short_name
            .map(|n| qualify(self.ns.as_deref(), &n))
            .map(|n| n.to_lowercase());
    }

    fn exit_static_scope(&mut self) {
        self.class = self.saved_class.take();
    }

    /// Classifies one assignment target. Unwraps array access / parentheses
    /// (`self::$x[0] = ...`, `(self::$x) = ...`) to the underlying expression;
    /// anything that is not a static-property access (variables, object
    /// properties, references) is ignored.
    fn classify_assign_target(&mut self, target: &Expr) {
        let mut t = target;
        loop {
            match &t.kind {
                ExprKind::ArrayAccess(a) => t = &a.array,
                ExprKind::Parenthesized(p) => t = p,
                ExprKind::StaticPropertyAccess(sp) => {
                    let prop = single_name(&sp.member);
                    match (prop, self.resolve_class(&sp.class)) {
                        (Some(prop), Some(class)) => {
                            self.out.writes.insert((class, prop.to_lowercase()));
                        }
                        _ => self.out.unresolved = true,
                    }
                    return;
                }
                // Variables, object properties, references: not static writes.
                _ => return,
            }
        }
    }

    /// Resolves the class side of a static access to a lowercased FQCN. In the
    /// owned AST a class reference is a single `Identifier` string (possibly
    /// backslash-qualified, possibly leading-backslash-absolute). `self::` /
    /// `static::` resolve to the enclosing class-like declaration; `parent::`
    /// and single-segment names (possible `use` aliases) are unresolvable;
    /// multi-segment relative names resolve against the current namespace.
    fn resolve_class(&self, class: &Expr) -> Option<String> {
        let id = match &class.kind {
            ExprKind::Identifier(s) => s.as_ref(),
            _ => return None,
        };
        let absolute = id.starts_with('\\');
        let trimmed = id.trim_start_matches('\\');
        let lower = trimmed.to_lowercase();
        let first = lower.split('\\').next().unwrap_or("");
        let segments = lower.split('\\').filter(|p| !p.is_empty()).count();
        match first {
            "self" | "static" => self.class.clone(),
            "parent" => None,
            _ => {
                if segments <= 1 {
                    return None;
                }
                let full = if absolute {
                    trimmed.to_string()
                } else {
                    qualify(self.ns.as_deref(), trimmed)
                };
                Some(full.to_lowercase())
            }
        }
    }
}

/// Lowercases a qualified name, preserving backslash separators.
fn lower_name(name: &Name) -> String {
    name.parts
        .iter()
        .map(|p| p.as_ref())
        .collect::<Vec<_>>()
        .join("\\")
        .to_lowercase()
}

/// Qualifies a class name against a lowercased namespace.
fn qualify(ns: Option<&str>, class: &str) -> String {
    match ns {
        Some(ns) if !ns.is_empty() => format!("{ns}\\{class}"),
        _ => class.to_string(),
    }
}

/// The identifier of a member expression that is a plain single-segment
/// `Identifier` — the property name of a static access. `None` for anything
/// else (qualified names, variables, `${$x}`), which the caller treats as
/// unresolvable.
fn single_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Identifier(s) if !s.contains('\\') => Some(s.as_ref()),
        _ => None,
    }
}

/// The type of a *literal* expression: integer, float, string, boolean,
/// `null`, or array literal. Non-literal expressions (calls, constant
/// fetches, casts, `new`, variables) yield `None` — the caller degrades those
/// sub-parts to `mixed` rather than failing the whole initializer.
pub(super) fn literal_type(expr: &Expr) -> Option<Type> {
    match &expr.kind {
        ExprKind::Int(_) => Some(Type::int()),
        ExprKind::Float(_) => Some(Type::float()),
        ExprKind::String(_) => Some(Type::string()),
        ExprKind::Bool(_) => Some(Type::bool()),
        ExprKind::Null => Some(Type::null()),
        ExprKind::Array(elems) => Some(literal_array(elems)),
        _ => None,
    }
}

/// The type of a literal array initializer.
///
/// Key kinds follow PHP's rules: a positional element has an integer key
/// (last explicit integer key + 1), explicit keys keep their scalar kind, and
/// `...$x` unpacking or a non-literal key degrades that key to `array-key`
/// (`int|string`). All-integer keys yield `list<V>`; otherwise `array<K, V>`.
/// The value is the union of the element literal types, degrading any
/// non-literal value to `mixed`. A positional list is never built as an open
/// keyed array: consumers of `array_keys()` on a list read the `int` key kind.
fn literal_array(elems: &[ArrayElement]) -> Type {
    if elems.iter().any(|e| e.unpack) {
        // `...$x` unpacking: keys and values are unknown.
        return bare_array();
    }
    let mut all_int_keys = true;
    let mut key: Option<Type> = None;
    let mut value = Type::mixed();
    for e in elems {
        let e_key = match &e.key {
            None => Type::int(), // positional elements carry integer keys
            Some(k) => literal_type(k).unwrap_or_else(Type::array_key),
        };
        if !is_int_kind(&e_key) {
            all_int_keys = false;
        }
        key = Some(match key {
            None => e_key,
            Some(acc) => merge_keys(&acc, &e_key),
        });
        let v = literal_type(&e.value).unwrap_or_else(Type::mixed);
        value = Type::merge(&value, &v);
    }
    match key {
        None => Type::single(Atomic::TList {
            value: Box::new(value), // `[]` — the empty list
        }),
        Some(k) => {
            if all_int_keys && is_int_kind(&k) {
                Type::single(Atomic::TList {
                    value: Box::new(value),
                })
            } else {
                Type::single(Atomic::TArray {
                    key: Box::new(k),
                    value: Box::new(value),
                })
            }
        }
    }
}

/// Merges two key types, collapsing same-kind duplicates and degrading to
/// `array-key` when integer and string keys are mixed (a key already degraded
/// to `array-key` stays `array-key`).
fn merge_keys(a: &Type, b: &Type) -> Type {
    if (is_int_kind(a) && is_int_kind(b)) || (is_string_kind(a) && is_string_kind(b)) {
        a.clone()
    } else {
        Type::array_key()
    }
}

fn is_int_kind(t: &Type) -> bool {
    t.types.len() == 1 && matches!(&t.types[0], Atomic::TInt)
}

fn is_string_kind(t: &Type) -> bool {
    t.types.len() == 1 && matches!(&t.types[0], Atomic::TString)
}

fn is_mixed_kind(t: &Type) -> bool {
    t.types.len() == 1 && matches!(&t.types[0], Atomic::TMixed)
}

/// The bare `array` type — `array<int|string, mixed>` — as produced by
/// `Type::array()`.
fn bare_array() -> Type {
    Type::single(Atomic::TArray {
        key: Box::new(Type::array_key()),
        value: Box::new(Type::mixed()),
    })
}

/// Whether a declared type is exactly bare `array` (the display rule of
/// [`mir_types`]: a single `TArray` whose key is `array-key` or `mixed` and
/// whose value is `mixed`). Declared shapes like `array<string, mixed>` do
/// *not* match.
fn is_bare_array(t: &Type) -> bool {
    if t.types.len() != 1 {
        return false;
    }
    let Atomic::TArray { key, value } = &t.types[0] else {
        return false;
    };
    (key.is_array_key() || is_mixed_kind(key)) && is_mixed_kind(value)
}

/// Refines a declared bare `array` property type into its literal initializer
/// type (see module docs for the soundness gate). Returns the literal type
/// when every gate passes; otherwise returns `declared` unchanged.
pub(super) fn refine_static_array_default(
    declared: Option<Type>,
    default: &Option<Type>,
    writes: &StaticWrites,
    fqcn: &str,
    prop: &str,
    is_private: bool,
    is_static: bool,
) -> Option<Type> {
    let refined = default.as_ref().filter(|t| !t.is_mixed());
    let can_refine = is_private
        && is_static
        && !writes.unresolved
        && refined.is_some()
        && declared.as_ref().is_some_and(is_bare_array)
        && !writes
            .writes
            .contains(&(fqcn.to_lowercase(), prop.to_lowercase()));
    if can_refine {
        Some(refined.unwrap().clone())
    } else {
        declared
    }
}
