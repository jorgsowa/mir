//! Definition collector.
//!
//! Visits every top-level declaration in the AST and produces a `StubSlice`
//! containing class, function, and constant signatures. No type inference
//! happens here.

use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;

use std::ops::ControlFlow;

use php_ast::ast::Visibility as AstVisibility;
use php_ast::owned::visitor::{walk_owned_program, walk_owned_stmt, OwnedVisitor};
use php_ast::owned::{Program, StmtKind};

use crate::parser::{name_to_string_owned, type_from_hint_owned};
use crate::php_version::PhpVersion;
use mir_codebase::definitions::{
    wrap_return_type, wrap_template_bound, Assertion, DeclaredParam, MethodDef, PropertyDef,
    StubSlice, TemplateParam, Visibility,
};
use mir_issues::{Issue, IssueBuffer};
use mir_types::{Atomic, Location, Name, Type};

mod annotation;
mod class;
mod r#enum;
mod function;
mod interface;
mod resolution;
mod r#trait;
mod version_attrs;

// ---------------------------------------------------------------------------
// Profiling counters for scalar type frequency
// ---------------------------------------------------------------------------

pub(crate) static SCALAR_PARAM_COUNT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static COMPLEX_PARAM_COUNT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static PARAM_WITH_DEFAULT: AtomicUsize = AtomicUsize::new(0);

/// Check if a Type is a simple scalar type (for profiling).
fn is_simple_scalar(u: &Type) -> bool {
    if u.possibly_undefined || u.from_docblock || u.types.len() != 1 {
        return false;
    }
    use mir_types::atomic::Atomic;
    matches!(
        &u.types[0],
        Atomic::TString
            | Atomic::TInt
            | Atomic::TFloat
            | Atomic::TIntegralFloat
            | Atomic::TBool
            | Atomic::TMixed
            | Atomic::TNull
            | Atomic::TVoid
            | Atomic::TNever
    )
}

/// Returns `true` when the native PHP hint is a single concrete scalar (bool/int/float/string)
/// whose scalar family is completely absent from the docblock type.
///
/// Used at collection time (no DB needed) to detect `@param int $x` + `bool $x` style
/// contradictions where the docblock has a *different* scalar family than the hint.
/// In that case the PHP hint is the runtime truth and should take precedence.
///
/// Does NOT fire when the docblock is a refinement of the hint (e.g. `positive-int` + `int`
/// hint, or `non-empty-string` + `string` hint): a refinement always contains atoms from the
/// same family, so `docblock_contains_hint_family` would be true and this returns false.
fn in_bool_family(a: &mir_types::atomic::Atomic) -> bool {
    use mir_types::atomic::Atomic;
    matches!(a, Atomic::TBool | Atomic::TTrue | Atomic::TFalse)
}
fn in_int_family(a: &mir_types::atomic::Atomic) -> bool {
    use mir_types::atomic::Atomic;
    matches!(
        a,
        Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TPositiveInt
            | Atomic::TNegativeInt
            | Atomic::TNonNegativeInt
    )
}
fn in_float_family(a: &mir_types::atomic::Atomic) -> bool {
    use mir_types::atomic::Atomic;
    matches!(a, Atomic::TFloat | Atomic::TLiteralFloat(_, _))
}
fn in_string_family(a: &mir_types::atomic::Atomic) -> bool {
    use mir_types::atomic::Atomic;
    matches!(
        a,
        Atomic::TString
            | Atomic::TLiteralString(_)
            | Atomic::TClassString(_)
            | Atomic::TInterfaceString(_)
            | Atomic::TNumericString
    )
}
fn is_any_scalar_family(a: &mir_types::atomic::Atomic) -> bool {
    in_bool_family(a) || in_int_family(a) || in_float_family(a) || in_string_family(a)
}

/// When `native` is a single concrete scalar (int/string/bool/float), returns
/// the family-membership check for that scalar's family. `None` when native
/// isn't such a type (no family-based conflict is detectable).
fn native_scalar_family(native: &Type) -> Option<fn(&mir_types::atomic::Atomic) -> bool> {
    if native.types.len() != 1 {
        return None;
    }
    Some(match &native.types[0] {
        a if in_bool_family(a) => in_bool_family,
        a if in_int_family(a) => in_int_family,
        a if in_float_family(a) => in_float_family,
        a if in_string_family(a) => in_string_family,
        _ => return None,
    })
}

pub(crate) fn native_hint_wins_over_docblock_scalar(native: &Type, doc: &Type) -> bool {
    if doc.types.is_empty() {
        return false;
    }
    let Some(family_check) = native_scalar_family(native) else {
        return false;
    };
    // Docblock must contain ONLY scalar atoms from other families (no mixed/null/object that
    // could be a union refinement, and none from the hint's own family).
    doc.types
        .iter()
        .all(|a| is_any_scalar_family(a) && !family_check(a))
}

/// When `native` is a concrete scalar and `doc` contains scalar atoms from a
/// DIFFERENT family, those atoms describe a value the native hint can never
/// actually hold at runtime — PHP enforces the native hint, so it's the
/// runtime truth. Strips such foreign atoms from `doc` (keeping any
/// same-family or non-scalar atoms, e.g. `null`, untouched), falling back to
/// `native` entirely when nothing scalar-compatible survives. Returns `doc`
/// unchanged when `native` isn't a single concrete scalar (no conflict is
/// detectable this way) or when nothing needed stripping.
pub(crate) fn resolve_docblock_scalar_conflict(native: &Type, doc: Type) -> Type {
    let Some(family_check) = native_scalar_family(native) else {
        return doc;
    };
    let has_foreign_scalar = doc
        .types
        .iter()
        .any(|a| is_any_scalar_family(a) && !family_check(a));
    if !has_foreign_scalar {
        return doc;
    }
    let mut filtered = Type::empty();
    filtered.from_docblock = doc.from_docblock;
    filtered.possibly_undefined = doc.possibly_undefined;
    for a in doc.types.iter() {
        if !is_any_scalar_family(a) || family_check(a) {
            filtered.add_type(a.clone());
        }
    }
    if filtered.types.is_empty() {
        native.clone()
    } else {
        filtered
    }
}

/// Returns true for PHP built-in type keywords and Psalm pseudo-types that must never be
/// namespace-qualified, even when they appear as TNamedObject (e.g. inside generic params).
fn is_php_builtin_type(name: &str) -> bool {
    matches!(
        name,
        "array"
            | "bool"
            | "callable"
            | "false"
            | "float"
            | "int"
            | "iterable"
            | "list"
            | "mixed"
            | "never"
            | "null"
            | "object"
            | "parent"
            | "positive-int"
            | "scalar"
            | "self"
            | "static"
            | "string"
            | "true"
            | "void"
            | "class-string"
            | "int-mask"
            | "int-mask-of"
            | "key-of"
            | "lowercase-string"
            | "negative-int"
            | "non-empty-array"
            | "non-empty-list"
            | "non-empty-string"
            | "non-falsy-string"
            | "numeric-string"
            | "truthy-string"
            | "value-of"
    )
}

/// Print profiling statistics for type collection.
pub(crate) fn print_collector_stats() {
    let scalar = SCALAR_PARAM_COUNT.load(Relaxed);
    let complex = COMPLEX_PARAM_COUNT.load(Relaxed);
    let with_default = PARAM_WITH_DEFAULT.load(Relaxed);
    let total = scalar + complex;
    let scalar_pct = if total > 0 {
        (scalar as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    eprintln!("  [collector stats]");
    eprintln!("    scalar params:        {} ({:.1}%)", scalar, scalar_pct);
    eprintln!("    complex params:       {}", complex);
    eprintln!("    params with default:  {}", with_default);
}

// ---------------------------------------------------------------------------
// Constant value inference
// ---------------------------------------------------------------------------

/// Infer the type of a constant value from its AST expression (owned AST).
/// This handles literal values like integers, strings, etc. used in define().
pub(super) fn infer_const_value(expr_kind: &php_ast::owned::ExprKind) -> Option<Type> {
    use php_ast::ast::{BinaryOp, UnaryPrefixOp};

    match expr_kind {
        php_ast::owned::ExprKind::Int(i) => Some(Type::single(Atomic::TLiteralInt(*i))),
        php_ast::owned::ExprKind::String(s) => {
            Some(Type::single(Atomic::TLiteralString(Arc::from(&**s))))
        }
        php_ast::owned::ExprKind::Float(_f) => Some(Type::single(Atomic::TFloat)),
        php_ast::owned::ExprKind::Bool(_b) => Some(Type::single(Atomic::TBool)),
        php_ast::owned::ExprKind::Null => Some(Type::single(Atomic::TNull)),
        // For unary expressions like -1, try to evaluate them
        php_ast::owned::ExprKind::UnaryPrefix(u) => match u.op {
            UnaryPrefixOp::Negate => {
                if let php_ast::owned::ExprKind::Int(i) = &u.operand.kind {
                    Some(Type::single(Atomic::TLiteralInt(-i)))
                } else {
                    None
                }
            }
            UnaryPrefixOp::Plus => {
                if let php_ast::owned::ExprKind::Int(i) = &u.operand.kind {
                    Some(Type::single(Atomic::TLiteralInt(*i)))
                } else {
                    None
                }
            }
            _ => None,
        },
        php_ast::owned::ExprKind::Parenthesized(inner) => infer_const_value(&inner.kind),
        // Idiomatic bitflag declarations (`const FLAG_A = 1 << 0;`) and other
        // literal-int arithmetic. Only evaluated when both operands are
        // themselves literal ints, so `self::OTHER_CONST | 1` still falls
        // through to `None` rather than guessing.
        php_ast::owned::ExprKind::Binary(b) => {
            let as_int = |t: Type| -> Option<i64> {
                (t.types.len() == 1)
                    .then(|| match t.types[0] {
                        Atomic::TLiteralInt(n) => Some(n),
                        _ => None,
                    })
                    .flatten()
            };
            let l = as_int(infer_const_value(&b.left.kind)?)?;
            let r = as_int(infer_const_value(&b.right.kind)?)?;
            let result = match b.op {
                BinaryOp::BitwiseOr => l | r,
                BinaryOp::BitwiseAnd => l & r,
                BinaryOp::BitwiseXor => l ^ r,
                BinaryOp::ShiftLeft => l.checked_shl(u32::try_from(r).ok()?)?,
                BinaryOp::ShiftRight => l.checked_shr(u32::try_from(r).ok()?)?,
                BinaryOp::Add => l.checked_add(r)?,
                BinaryOp::Sub => l.checked_sub(r)?,
                BinaryOp::Mul => l.checked_mul(r)?,
                _ => return None,
            };
            Some(Type::single(Atomic::TLiteralInt(result)))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// DefinitionCollector
// ---------------------------------------------------------------------------

pub struct DefinitionCollector<'a> {
    slice: StubSlice,
    file: Arc<str>,
    source: &'a str,
    source_map: &'a php_rs_parser::source_map::SourceMap,
    namespace: Option<String>,
    /// `use` aliases: alias → FQCN. `UseKind::Normal` only — every consumer
    /// resolves a class/type/attribute/exception name, so a `use function`/
    /// `use const` alias must never appear here.
    use_aliases: FxHashMap<String, String>,
    issues: IssueBuffer,
    /// When `Some`, stub symbols annotated with `@since`/`@removed` are filtered
    /// against this target version. `None` disables filtering (user code).
    php_version: Option<PhpVersion>,
    /// The first namespace declaration seen in this file. Matches the semantics
    /// of `project.rs` which only records the first namespace per file.
    first_namespace: Option<String>,
    /// All `use` imports ever encountered in this file — every `UseKind`,
    /// unlike `use_aliases` — accumulated across all namespace blocks. Unlike
    /// `use_aliases`, this is never cleared or restored, so braced-namespace
    /// imports are not lost. Feeds `slice.imports` / `file_imports()`, which
    /// Pass-2 function-call resolution relies on for `use function` aliases.
    accumulated_imports: FxHashMap<String, String>,
    /// Subset of `accumulated_imports` containing only `UseKind::Normal`
    /// (class/interface/trait/enum) aliases — excludes `use function`/`use const`.
    /// Feeds `slice.class_imports`, consulted by class-name resolution so a
    /// function/constant import can't shadow a same-named class reference.
    accumulated_class_imports: FxHashMap<String, String>,
}

impl<'a> DefinitionCollector<'a> {
    pub fn new_for_slice(
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
    ) -> Self {
        let slice = StubSlice {
            file: Some(file.clone()),
            ..StubSlice::default()
        };
        Self {
            source_map,
            slice,
            file,
            source,
            namespace: None,
            use_aliases: FxHashMap::default(),
            issues: IssueBuffer::new(),
            php_version: None,
            first_namespace: None,
            accumulated_imports: FxHashMap::default(),
            accumulated_class_imports: FxHashMap::default(),
        }
    }

    /// Enable `@since`/`@removed` filtering against the given target PHP
    /// version. Used by the stub loader so that symbols introduced after, or
    /// removed at or before, the target version are not registered.
    pub fn with_php_version(mut self, version: PhpVersion) -> Self {
        self.php_version = Some(version);
        self
    }

    /// Returns `true` if a docblock's `@since`/`@removed` tags allow this
    /// symbol to exist at the configured target version. When no target is
    /// configured (user code), always returns `true`.
    fn version_allows(&self, doc: &crate::parser::ParsedDocblock) -> bool {
        match self.php_version {
            Some(v) => v.includes_symbol(doc.since.as_deref(), doc.removed.as_deref()),
            None => true,
        }
    }

    /// Whether a stub element (function/method/param) carrying
    /// `#[PhpStormStubsElementAvailable]` is available at the configured target
    /// version. Always `true` for user code (`php_version == None`) or when the
    /// attribute is absent. See [`version_attrs`].
    fn version_attr_available(&self, attrs: &[php_ast::owned::Attribute]) -> bool {
        match self.php_version {
            Some(v) => version_attrs::is_available(attrs, &self.use_aliases, v),
            None => true,
        }
    }

    /// The `#[LanguageLevelTypeAware]` type-string override for the target
    /// version, if any. `None` for user code, an absent attribute, or an empty
    /// (`default: ''`) resolution. Callers parse the string via
    /// [`parse_type_string`](crate::parser::docblock::parse_type_string).
    fn version_attr_type_string(&self, attrs: &[php_ast::owned::Attribute]) -> Option<String> {
        let v = self.php_version?;
        version_attrs::type_aware(attrs, &self.use_aliases, v)
    }

    fn parse_docblock_from_node(
        &self,
        doc_comment: Option<&php_ast::owned::Comment>,
    ) -> crate::parser::ParsedDocblock {
        doc_comment
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default()
    }

    /// Writes accumulated namespace and import data into `self.slice` so that
    /// `file_namespace()` and `file_imports()` can derive them via
    /// `collect_file_definitions`. Called at the end of `collect_slice`.
    fn finalize_slice(&mut self) {
        if let Some(ns) = self.first_namespace.take() {
            self.slice.namespace = Some(Arc::from(ns.as_str()));
        }
        if !self.accumulated_imports.is_empty() {
            // Convert collector's String-keyed map into the storage shape:
            // Arc<FxHashMap<Name, Name>>. `Name::new` interns each string
            // via the global ustr pool once per unique alias/FQCN.
            let raw = std::mem::take(&mut self.accumulated_imports);
            let mut interned: FxHashMap<mir_types::Name, mir_types::Name> =
                FxHashMap::with_capacity_and_hasher(raw.len(), Default::default());
            for (alias, fqcn) in raw {
                interned.insert(mir_types::Name::new(&alias), mir_types::Name::new(&fqcn));
            }
            self.slice.imports = Arc::new(interned);
        }
        if !self.accumulated_class_imports.is_empty() {
            let raw = std::mem::take(&mut self.accumulated_class_imports);
            let mut interned: FxHashMap<mir_types::Name, mir_types::Name> =
                FxHashMap::with_capacity_and_hasher(raw.len(), Default::default());
            for (alias, fqcn) in raw {
                interned.insert(mir_types::Name::new(&alias), mir_types::Name::new(&fqcn));
            }
            self.slice.class_imports = Arc::new(interned);
        }
    }

    pub fn collect_slice(mut self, program: &Program) -> (StubSlice, Vec<Issue>) {
        let _ = self.visit_program(program);
        self.finalize_slice();
        (self.slice, self.issues.into_all_issues())
    }

    // -----------------------------------------------------------------------
    // FQCN resolution helpers
    // -----------------------------------------------------------------------
    // Type Resolution (delegating to resolution module)
    // -----------------------------------------------------------------------

    fn resolve_name(&self, name: &str) -> String {
        resolution::resolve_name(name, &self.namespace, &self.use_aliases)
    }

    /// Compute the FQCN a class/interface/trait/enum *declaration* establishes
    /// for its own short name: `current_namespace \ short_name`, never run
    /// through `use`-alias substitution. A declaration names a new symbol, it
    /// doesn't reference an existing one — unlike `resolve_name`, which must
    /// consult `use_aliases` because callers pass it names being *referenced*
    /// (`extends`, `implements`, type hints, ...). Using `resolve_name` here
    /// misfires when the short name collides with a `use function`/`use
    /// const` alias of the same spelling (legal in PHP, since functions and
    /// constants live in a separate symbol table from classes): the
    /// declaration would be silently registered under the alias's target
    /// FQCN instead of its own namespace.
    fn declared_fqn(&self, short_name: &str) -> String {
        match &self.namespace {
            Some(ns) => format!("{ns}\\{short_name}"),
            None => short_name.to_string(),
        }
    }

    fn resolve_type_name(&self, name: &str, full_qualify: bool) -> mir_types::Name {
        resolution::resolve_type_name(name, full_qualify, &self.namespace, &self.use_aliases)
    }

    fn fill_self_static_parent(union: Type, class_fqcn: &str) -> Type {
        resolution::fill_self_static_parent(union, class_fqcn)
    }

    fn resolve_union(&self, union: Type) -> Type {
        resolution::resolve_union(union, &self.namespace, &self.use_aliases)
    }

    fn resolve_union_doc(&self, union: Type) -> Type {
        resolution::resolve_union_doc(union, &self.namespace, &self.use_aliases)
    }

    fn resolve_union_doc_with_aliases(
        &self,
        union: Type,
        aliases: &FxHashMap<String, Type>,
    ) -> Type {
        resolution::resolve_union_doc_with_aliases(
            union,
            aliases,
            &self.namespace,
            &self.use_aliases,
        )
    }

    fn resolve_union_opt(&self, opt: Option<Type>) -> Option<Type> {
        resolution::resolve_union_opt(opt, &self.namespace, &self.use_aliases)
    }

    fn resolve_union_doc_with_templates(
        &self,
        union: Type,
        template_names: &rustc_hash::FxHashSet<String>,
        defining_entity: &str,
        template_params: &[TemplateParam],
    ) -> Type {
        let mut result = Type::empty();
        result.possibly_undefined = union.possibly_undefined;
        result.from_docblock = union.from_docblock;
        for atomic in union.types {
            match &atomic {
                mir_types::Atomic::TNamedObject { fqcn, type_params }
                    if type_params.is_empty() && template_names.contains(fqcn.as_ref()) =>
                {
                    // Find the bound for this template parameter
                    let bound = template_params
                        .iter()
                        .find(|tp| tp.name.as_ref() == fqcn.as_ref())
                        .and_then(|tp| tp.bound.as_deref().cloned())
                        .unwrap_or_else(Type::mixed);

                    // This is a template parameter reference
                    result.add_type(mir_types::Atomic::TTemplateParam {
                        name: *fqcn,
                        as_type: Box::new(bound),
                        defining_entity: defining_entity.into(),
                    });
                }
                // A generic class like ObjectProphecy<T>: the outer class name must be
                // FQN-qualified (it is a real class, not a template), and type_params are
                // recursed through this function so template names inside (e.g. T) are
                // properly converted to TTemplateParam.
                // Guard: PHP built-in pseudo-types (array, iterable, callable, …) can appear
                // as TNamedObject with type params in some docblock parse paths; do not
                // namespace-qualify those — fall through to resolve_union_doc.
                mir_types::Atomic::TNamedObject { fqcn, type_params }
                    if !type_params.is_empty() && !is_php_builtin_type(fqcn.as_ref()) =>
                {
                    let resolved_fqcn = resolution::resolve_type_name(
                        fqcn.as_ref(),
                        true,
                        &self.namespace,
                        &self.use_aliases,
                    );
                    let new_params: Vec<Type> = type_params
                        .iter()
                        .map(|p| {
                            self.resolve_union_doc_with_templates(
                                p.clone(),
                                template_names,
                                defining_entity,
                                template_params,
                            )
                        })
                        .collect();
                    result.add_type(mir_types::Atomic::TNamedObject {
                        fqcn: resolved_fqcn,
                        type_params: mir_types::union::vec_to_type_params(new_params),
                    });
                }
                // Bare non-template class name (empty type_params, not a template param):
                // FQN-qualify it so same-namespace class references in docblocks are stored
                // with their full path. PHP built-in type keywords (array, list, callable, …)
                // are excluded — they must not be namespace-qualified even if the docblock
                // parser emits them as TNamedObject.
                mir_types::Atomic::TNamedObject { fqcn, .. }
                    if !is_php_builtin_type(fqcn.as_ref()) =>
                {
                    let resolved_fqcn = resolution::resolve_type_name(
                        fqcn.as_ref(),
                        true,
                        &self.namespace,
                        &self.use_aliases,
                    );
                    result.add_type(mir_types::Atomic::TNamedObject {
                        fqcn: resolved_fqcn,
                        type_params: mir_types::union::empty_type_params(),
                    });
                }
                // Intersection bound like `Type&Named`: recurse into each part so every
                // class name inside is FQN-qualified and template references are converted.
                mir_types::Atomic::TIntersection { parts } => {
                    let new_parts: Vec<Type> = parts
                        .iter()
                        .map(|p| {
                            self.resolve_union_doc_with_templates(
                                p.clone(),
                                template_names,
                                defining_entity,
                                template_params,
                            )
                        })
                        .collect();
                    result.add_type(mir_types::Atomic::TIntersection {
                        parts: mir_types::union::vec_to_type_params(new_parts),
                    });
                }
                // Array types: recurse into key and value with template awareness so that
                // bare template names like `L` inside `array<int, L>` are converted to
                // TTemplateParam rather than left as unresolved TNamedObject references.
                mir_types::Atomic::TArray { key, value } => {
                    result.add_type(mir_types::Atomic::TArray {
                        key: Box::new(self.resolve_union_doc_with_templates(
                            *key.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        )),
                        value: Box::new(self.resolve_union_doc_with_templates(
                            *value.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        )),
                    });
                }
                mir_types::Atomic::TNonEmptyArray { key, value } => {
                    result.add_type(mir_types::Atomic::TNonEmptyArray {
                        key: Box::new(self.resolve_union_doc_with_templates(
                            *key.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        )),
                        value: Box::new(self.resolve_union_doc_with_templates(
                            *value.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        )),
                    });
                }
                mir_types::Atomic::TList { value } => {
                    result.add_type(mir_types::Atomic::TList {
                        value: Box::new(self.resolve_union_doc_with_templates(
                            *value.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        )),
                    });
                }
                mir_types::Atomic::TNonEmptyList { value } => {
                    result.add_type(mir_types::Atomic::TNonEmptyList {
                        value: Box::new(self.resolve_union_doc_with_templates(
                            *value.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        )),
                    });
                }
                // Conditional return type: recurse into subject and both branches with the
                // same template context so class names and template references inside them
                // are resolved correctly.
                mir_types::Atomic::TConditional { data } => {
                    result.add_type(mir_types::Atomic::TConditional {
                        data: Box::new(mir_types::atomic::ConditionalData {
                            param_name: data.param_name,
                            subject: self.resolve_union_doc_with_templates(
                                data.subject.clone(),
                                template_names,
                                defining_entity,
                                template_params,
                            ),
                            if_true: self.resolve_union_doc_with_templates(
                                data.if_true.clone(),
                                template_names,
                                defining_entity,
                                template_params,
                            ),
                            if_false: self.resolve_union_doc_with_templates(
                                data.if_false.clone(),
                                template_names,
                                defining_entity,
                                template_params,
                            ),
                        }),
                    });
                }
                // Closure/callable param & return types: recurse with template awareness so
                // a bare template name used inside `Closure(T): R` (e.g. a higher-order
                // function's predicate/mapper param) is converted to TTemplateParam instead
                // of falling through to resolve_union_doc, which has no template context and
                // would leave it as an unresolved bare class-like reference to "T".
                mir_types::Atomic::TClosure { data } => {
                    let new_params = data
                        .params
                        .iter()
                        .map(|p| {
                            let mut p = p.clone();
                            p.ty = p.ty.as_ref().map(|t| {
                                mir_types::compact::SimpleType::from_union(
                                    self.resolve_union_doc_with_templates(
                                        t.to_union(),
                                        template_names,
                                        defining_entity,
                                        template_params,
                                    ),
                                )
                            });
                            p
                        })
                        .collect();
                    result.add_type(mir_types::Atomic::TClosure {
                        data: Box::new(mir_types::atomic::ClosureData {
                            params: new_params,
                            return_type: self.resolve_union_doc_with_templates(
                                data.return_type.clone(),
                                template_names,
                                defining_entity,
                                template_params,
                            ),
                            this_type: data.this_type.clone(),
                        }),
                    });
                }
                mir_types::Atomic::TCallable {
                    params,
                    return_type,
                } => {
                    let new_params = params.as_ref().map(|ps| {
                        ps.iter()
                            .map(|p| {
                                let mut p = p.clone();
                                p.ty = p.ty.as_ref().map(|t| {
                                    mir_types::compact::SimpleType::from_union(
                                        self.resolve_union_doc_with_templates(
                                            t.to_union(),
                                            template_names,
                                            defining_entity,
                                            template_params,
                                        ),
                                    )
                                });
                                p
                            })
                            .collect()
                    });
                    let new_return_type = return_type.as_deref().map(|t| {
                        Box::new(self.resolve_union_doc_with_templates(
                            t.clone(),
                            template_names,
                            defining_entity,
                            template_params,
                        ))
                    });
                    result.add_type(mir_types::Atomic::TCallable {
                        params: new_params,
                        return_type: new_return_type,
                    });
                }
                _ => {
                    let resolved_union = self.resolve_union_doc(Type::single(atomic.clone()));
                    for resolved_atomic in resolved_union.types {
                        result.add_type(resolved_atomic);
                    }
                }
            }
        }
        result
    }

    /// Post-resolution template substitution for method params.
    ///
    /// `resolve_union_doc` / `resolve_union_doc_with_aliases` use `full_qualify=false`
    /// so bare names like `Closure` or `Countable` stay bare (correct behavior for params).
    /// But template param names (e.g. `TRelatedModel`) also stay bare — they need a second
    /// pass to become `TTemplateParam`. This function does ONLY that conversion without
    /// touching qualification, so it is safe to call after `resolve_union_doc`.
    fn substitute_template_params(
        &self,
        ty: Type,
        template_names: &rustc_hash::FxHashSet<String>,
        template_params: &[TemplateParam],
        defining_entity: &str,
    ) -> Type {
        let mut result = Type::empty();
        result.possibly_undefined = ty.possibly_undefined;
        result.from_docblock = ty.from_docblock;
        for atomic in ty.types {
            match &atomic {
                mir_types::Atomic::TNamedObject { fqcn, type_params }
                    if type_params.is_empty() && template_names.contains(fqcn.as_ref()) =>
                {
                    let bound = template_params
                        .iter()
                        .find(|tp| tp.name.as_ref() == fqcn.as_ref())
                        .and_then(|tp| tp.bound.as_deref().cloned())
                        .unwrap_or_else(Type::mixed);
                    result.add_type(mir_types::Atomic::TTemplateParam {
                        name: *fqcn,
                        as_type: Box::new(bound),
                        defining_entity: defining_entity.into(),
                    });
                }
                mir_types::Atomic::TNamedObject { fqcn, type_params }
                    if !type_params.is_empty() =>
                {
                    let new_params: Vec<Type> = type_params
                        .iter()
                        .map(|p| {
                            self.substitute_template_params(
                                p.clone(),
                                template_names,
                                template_params,
                                defining_entity,
                            )
                        })
                        .collect();
                    result.add_type(mir_types::Atomic::TNamedObject {
                        fqcn: *fqcn,
                        type_params: mir_types::union::vec_to_type_params(new_params),
                    });
                }
                mir_types::Atomic::TIntersection { parts } => {
                    let new_parts: Vec<Type> = parts
                        .iter()
                        .map(|p| {
                            self.substitute_template_params(
                                p.clone(),
                                template_names,
                                template_params,
                                defining_entity,
                            )
                        })
                        .collect();
                    result.add_type(mir_types::Atomic::TIntersection {
                        parts: mir_types::union::vec_to_type_params(new_parts),
                    });
                }
                mir_types::Atomic::TArray { key, value } => {
                    result.add_type(mir_types::Atomic::TArray {
                        key: Box::new(self.substitute_template_params(
                            *key.clone(),
                            template_names,
                            template_params,
                            defining_entity,
                        )),
                        value: Box::new(self.substitute_template_params(
                            *value.clone(),
                            template_names,
                            template_params,
                            defining_entity,
                        )),
                    });
                }
                mir_types::Atomic::TNonEmptyArray { key, value } => {
                    result.add_type(mir_types::Atomic::TNonEmptyArray {
                        key: Box::new(self.substitute_template_params(
                            *key.clone(),
                            template_names,
                            template_params,
                            defining_entity,
                        )),
                        value: Box::new(self.substitute_template_params(
                            *value.clone(),
                            template_names,
                            template_params,
                            defining_entity,
                        )),
                    });
                }
                mir_types::Atomic::TList { value } => {
                    result.add_type(mir_types::Atomic::TList {
                        value: Box::new(self.substitute_template_params(
                            *value.clone(),
                            template_names,
                            template_params,
                            defining_entity,
                        )),
                    });
                }
                mir_types::Atomic::TClosure { data } => {
                    let new_params = data
                        .params
                        .iter()
                        .map(|p| {
                            let mut p = p.clone();
                            p.ty = p.ty.as_ref().map(|t| {
                                mir_types::compact::SimpleType::from_union(
                                    self.substitute_template_params(
                                        t.to_union(),
                                        template_names,
                                        template_params,
                                        defining_entity,
                                    ),
                                )
                            });
                            p
                        })
                        .collect();
                    result.add_type(mir_types::Atomic::TClosure {
                        data: Box::new(mir_types::atomic::ClosureData {
                            params: new_params,
                            return_type: self.substitute_template_params(
                                data.return_type.clone(),
                                template_names,
                                template_params,
                                defining_entity,
                            ),
                            this_type: data.this_type.clone(),
                        }),
                    });
                }
                mir_types::Atomic::TCallable {
                    params,
                    return_type,
                } => {
                    let new_params = params.as_ref().map(|ps| {
                        ps.iter()
                            .map(|p| {
                                let mut p = p.clone();
                                p.ty = p.ty.as_ref().map(|t| {
                                    mir_types::compact::SimpleType::from_union(
                                        self.substitute_template_params(
                                            t.to_union(),
                                            template_names,
                                            template_params,
                                            defining_entity,
                                        ),
                                    )
                                });
                                p
                            })
                            .collect()
                    });
                    let new_return_type = return_type.as_deref().map(|t| {
                        Box::new(self.substitute_template_params(
                            t.clone(),
                            template_names,
                            template_params,
                            defining_entity,
                        ))
                    });
                    result.add_type(mir_types::Atomic::TCallable {
                        params: new_params,
                        return_type: new_return_type,
                    });
                }
                _ => result.add_type(atomic),
            }
        }
        result
    }

    fn build_assertions(&self, doc: &crate::parser::ParsedDocblock) -> Vec<Assertion> {
        annotation::build_assertions(doc, |u| self.resolve_union_doc(u))
    }

    fn location(&self, start: u32, end: u32) -> Location {
        let src = self.source;
        let start_off = start as usize;
        let line_start = src[..start_off].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line = self.source_map.offset_to_line_col(start).line + 1;
        let col_start = src[line_start..start_off].chars().count() as u16;

        let end_off = (end as usize).min(src.len());
        let end_line_start = src[..end_off].rfind('\n').map(|p| p + 1).unwrap_or(0);
        let line_end = self.source_map.offset_to_line_col(end_off as u32).line + 1;
        let col_end = src[end_line_start..end_off].chars().count() as u16;

        Location::new(self.file.clone(), line, line_end, col_start, col_end)
    }

    // -----------------------------------------------------------------------
    // Docblock issue emission
    // -----------------------------------------------------------------------

    fn emit_docblock_issues(&mut self, doc: &crate::parser::ParsedDocblock, span_start: u32) {
        annotation::emit_docblock_issues(
            doc,
            span_start,
            self.php_version,
            self.file.clone(),
            self.source_map,
            &mut self.issues,
        );
    }

    // -----------------------------------------------------------------------
    // Visibility conversion
    // -----------------------------------------------------------------------

    fn convert_visibility(v: Option<AstVisibility>) -> Visibility {
        match v {
            Some(AstVisibility::Public) | None => Visibility::Public,
            Some(AstVisibility::Protected) => Visibility::Protected,
            Some(AstVisibility::Private) => Visibility::Private,
        }
    }

    /// Substitute alias names in `union` with their pre-built definitions.
    /// Does not touch FQN resolution; that is left to the caller's resolution pass.
    fn expand_aliases_only(&self, union: Type, aliases: &FxHashMap<String, Type>) -> Type {
        if aliases.is_empty() {
            return union;
        }
        let from_docblock = union.from_docblock;
        let mut result = Type::empty();
        result.possibly_undefined = union.possibly_undefined;
        result.from_docblock = from_docblock;
        for atomic in union.types {
            match atomic {
                mir_types::Atomic::TNamedObject {
                    ref fqcn,
                    ref type_params,
                } if type_params.is_empty() => {
                    if let Some(alias_ty) = aliases.get(fqcn.as_ref()) {
                        result.merge_with(alias_ty);
                    } else {
                        result.add_type(atomic);
                    }
                }
                other => result.add_type(other),
            }
        }
        result
    }

    fn build_type_aliases(&self, doc: &crate::parser::ParsedDocblock) -> FxHashMap<String, Type> {
        let mut aliases = FxHashMap::default();
        for alias in &doc.type_aliases {
            if alias.name.is_empty() || alias.type_expr.is_empty() {
                continue;
            }
            let mut ty = crate::parser::docblock::parse_type_string(&alias.type_expr);
            ty.from_docblock = true;
            aliases.insert(alias.name.clone(), self.resolve_union_doc(ty));
        }

        // Resolve same-file @psalm-import-type declarations. Cross-file imports
        // stay in `pending_import_types` and are resolved after all slices are
        // injected.
        for import in &doc.import_types {
            if import.from_class.is_empty() {
                continue;
            }
            let from_resolved = self.resolve_type_name(import.from_class.as_str(), true);
            let resolved = self
                .slice
                .classes
                .iter()
                .find(|cls| cls.fqcn.as_ref() == from_resolved.as_ref())
                .and_then(|cls| cls.type_aliases.get(import.original.as_str()).cloned());
            if let Some(ty) = resolved {
                aliases.insert(import.local.clone(), ty);
            }
        }

        aliases
    }

    fn add_docblock_members(
        &self,
        doc: &crate::parser::ParsedDocblock,
        aliases: &FxHashMap<String, Type>,
        class_fqcn: &str,
        own_methods: &mut mir_codebase::definitions::MemberMap<Arc<MethodDef>>,
        own_properties: &mut mir_codebase::definitions::MemberMap<PropertyDef>,
        location: Option<Location>,
    ) {
        for prop in &doc.properties {
            if prop.name.is_empty() || own_properties.contains_key(prop.name.as_str()) {
                continue;
            }
            let ty = if prop.type_hint.is_empty() {
                None
            } else {
                let mut parsed = crate::parser::docblock::parse_type_string(&prop.type_hint);
                parsed.from_docblock = true;
                Some(self.resolve_union_doc_with_aliases(parsed, aliases))
            };
            own_properties.insert(
                Arc::from(prop.name.as_str()),
                PropertyDef {
                    name: Arc::from(prop.name.as_str()),
                    ty: mir_codebase::definitions::wrap_property_type(ty),
                    native_ty: None,
                    inferred_ty: None,
                    visibility: Visibility::Public,
                    is_static: false,
                    is_readonly: prop.read_only,
                    default: None,
                    location: location.clone(),
                    deprecated: None,
                    has_native_readonly: false,
                    // Magic `@property` declarations carry no PHP native type.
                    has_native_type: false,
                    from_docblock: true,
                },
            );
        }

        for method in &doc.methods {
            if method.name.is_empty() {
                continue;
            }
            let key = Arc::from(crate::util::php_ident_lowercase(&method.name).as_str());
            if own_methods.contains_key(&key) {
                continue;
            }
            let return_type_opt = if method.return_type.is_empty() {
                None
            } else {
                let mut parsed = crate::parser::docblock::parse_type_string(&method.return_type);
                parsed.from_docblock = true;
                Some(Self::fill_self_static_parent(
                    self.resolve_union_doc_with_aliases(parsed, aliases),
                    class_fqcn,
                ))
            };
            let params = method
                .params
                .iter()
                .map(|p| {
                    let ty = if p.type_hint.is_empty() {
                        None
                    } else {
                        let mut parsed = crate::parser::docblock::parse_type_string(&p.type_hint);
                        parsed.from_docblock = true;
                        Some(self.resolve_union_doc_with_aliases(parsed, aliases))
                    };
                    DeclaredParam {
                        name: Name::new(p.name.as_str()),
                        ty: mir_codebase::wrap_param_type(ty),
                        out_ty: None,
                        has_default: p.is_optional,
                        is_variadic: p.is_variadic,
                        is_byref: p.is_byref,
                        is_optional: p.is_optional,
                    }
                })
                .collect();
            own_methods.insert(
                key,
                Arc::new(MethodDef {
                    name: Arc::from(method.name.as_str()),
                    fqcn: Arc::from(class_fqcn),
                    params,
                    return_type: wrap_return_type(return_type_opt),
                    inferred_return_type: None,
                    visibility: Visibility::Public,
                    is_static: method.is_static,
                    is_abstract: false,
                    is_final: false,
                    is_constructor: false,
                    template_params: vec![],
                    assertions: vec![],
                    throws: vec![],
                    deprecated: None,
                    is_internal: false,
                    is_pure: false,
                    no_named_arguments: false,
                    is_override: false,
                    location: location.clone(),
                    docstring: None,
                    is_virtual: true,
                    taint_sink_params: vec![],
                    if_this_is: None,
                    self_out: None,
                    is_inherit_doc: false,
                    is_mutation_free: false,
                    is_external_mutation_free: false,
                }),
            );
        }
    }

    // -----------------------------------------------------------------------
    // Process statements
    // -----------------------------------------------------------------------

    fn process_stmts(&mut self, stmts: &[php_ast::owned::Stmt]) -> ControlFlow<()> {
        for stmt in stmts.iter() {
            self.visit_stmt(stmt)?;
        }
        ControlFlow::Continue(())
    }

    // -----------------------------------------------------------------------
    // Global variable registry
    // -----------------------------------------------------------------------

    /// Scan a single statement: if it is `global $x` with a preceding
    /// `/** @var Type $x */` docblock, register the type in the codebase.
    fn try_collect_global_var_annotation(&mut self, stmt: &php_ast::owned::Stmt) {
        let php_ast::owned::StmtKind::Global(vars) = &stmt.kind else {
            return;
        };
        let Some(doc_comment) = stmt.leading_doc_comment() else {
            return;
        };
        let parsed = crate::parser::DocblockParser::parse(&doc_comment.text);
        self.emit_docblock_issues(&parsed, stmt.span.start);
        let Some(var_type) = parsed.var_type else {
            return;
        };
        let resolved_ty = self.resolve_union_doc(var_type);

        for var in vars.iter() {
            if let php_ast::owned::ExprKind::Variable(raw_name) = &var.kind {
                let name = raw_name.trim_start_matches('$');
                // If @var specifies a variable name, only register when it matches.
                if let Some(ref ann_name) = parsed.var_name {
                    if ann_name != name {
                        continue;
                    }
                }
                self.slice
                    .global_vars
                    .push((Arc::from(name), resolved_ty.clone()));
            }
        }
    }

    /// Scan a list of statements and register any `@var`-annotated `global`
    /// declarations. Used for function bodies where the visitor does not recurse.
    fn scan_stmts_for_global_vars(&mut self, stmts: &[php_ast::owned::Stmt]) {
        for stmt in stmts.iter() {
            self.try_collect_global_var_annotation(stmt);
        }
    }
}

impl<'a> OwnedVisitor for DefinitionCollector<'a> {
    fn visit_program(&mut self, program: &Program) -> ControlFlow<()> {
        walk_owned_program(self, program)
    }

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Namespace(ns) => {
                let new_ns = ns.name.as_ref().map(name_to_string_owned);
                if self.first_namespace.is_none() {
                    self.first_namespace = new_ns.clone();
                }
                self.namespace = new_ns;
                match &ns.body {
                    php_ast::owned::NamespaceBody::Braced(stmts) => {
                        // Save and restore use aliases per namespace block
                        let saved_aliases = self.use_aliases.clone();
                        self.use_aliases.clear();
                        let flow = self.process_stmts(&stmts.stmts);
                        self.use_aliases = saved_aliases;
                        flow?;
                    }
                    php_ast::owned::NamespaceBody::Simple => {
                        // Simple namespace — affects all subsequent declarations
                    }
                }
            }

            StmtKind::Use(use_decl) => {
                use php_ast::ast::UseKind;
                for item in use_decl.uses.iter() {
                    let full_name = name_to_string_owned(&item.name)
                        .trim_start_matches('\\')
                        .to_string();
                    let alias = item
                        .alias
                        .as_deref()
                        .unwrap_or_else(|| full_name.rsplit('\\').next().unwrap_or(&full_name));
                    // `accumulated_imports` (→ `file_imports()`) keeps every kind: Pass 2
                    // function-call resolution (`call/function.rs`) relies on `use
                    // function` aliases showing up there. `use_aliases` and
                    // `accumulated_class_imports` (→ `file_class_imports()`) are
                    // Normal-only: every Pass-1 consumer of `use_aliases` resolves a
                    // class/type/attribute/exception name, so a `use function`/`use
                    // const` alias (including per-item overrides inside a grouped `use
                    // Foo\{Bar, function baz, const QUX}`) must never populate them —
                    // otherwise a type hint/`new`/`extends` reference sharing that short
                    // name would incorrectly resolve to the function/constant's FQN.
                    self.accumulated_imports
                        .insert(alias.to_string(), full_name.clone());
                    if item.kind.unwrap_or(use_decl.kind) == UseKind::Normal {
                        self.use_aliases
                            .insert(alias.to_string(), full_name.clone());
                        self.accumulated_class_imports
                            .insert(alias.to_string(), full_name);
                    }
                }
            }

            StmtKind::Function(decl) => {
                self.collect_function(decl, stmt.span);
            }

            StmtKind::Global(_) => {
                self.collect_global_stmt(stmt);
            }

            StmtKind::Class(decl) => {
                return self.collect_class(decl, stmt.span);
            }

            StmtKind::Interface(decl) => {
                return self.collect_interface(decl, stmt.span);
            }

            StmtKind::Trait(decl) => {
                return self.collect_trait(decl, stmt.span);
            }

            StmtKind::Enum(decl) => {
                self.collect_enum(decl, stmt.span);
            }

            StmtKind::Const(items) => {
                for item in items.iter() {
                    let const_doc = item
                        .doc_comment
                        .as_ref()
                        .map(|c| crate::parser::DocblockParser::parse(&c.text))
                        .unwrap_or_default();
                    let const_doc_span = item
                        .doc_comment
                        .as_ref()
                        .map(|c| c.span.start)
                        .unwrap_or(item.span.start);
                    self.emit_docblock_issues(&const_doc, const_doc_span);
                    if !self.version_allows(&const_doc) {
                        continue;
                    }
                    let name_str = item.name.as_deref().unwrap_or_default();
                    let fqn: Arc<str> = if let Some(ns) = &self.namespace {
                        format!("{}\\{}", ns, name_str).into()
                    } else {
                        Arc::from(name_str)
                    };
                    self.slice.constants.push((fqn, Type::mixed()));
                }
            }

            // Collect top-level define('NAME', value) calls as global constants.
            // phpstorm-stubs uses this form extensively in *_defines.php files.
            StmtKind::Expression(expr) => {
                if let php_ast::owned::ExprKind::FunctionCall(call) = &expr.kind {
                    if let php_ast::owned::ExprKind::Identifier(fn_name) = &call.name.kind {
                        if fn_name.eq_ignore_ascii_case("define") {
                            if let Some(name_arg) = call.args.first() {
                                if let php_ast::owned::ExprKind::String(name) = &name_arg.value.kind
                                {
                                    let define_doc = stmt
                                        .leading_doc_comment()
                                        .map(|c| crate::parser::DocblockParser::parse(&c.text))
                                        .unwrap_or_default();
                                    self.emit_docblock_issues(&define_doc, stmt.span.start);
                                    if self.version_allows(&define_doc) {
                                        let fqn: Arc<str> = Arc::from(&**name);
                                        // Try to infer the type of the constant value from the second argument
                                        let const_type = call
                                            .args
                                            .get(1)
                                            .and_then(|arg| infer_const_value(&arg.value.kind))
                                            .unwrap_or(Type::mixed());
                                        self.slice.constants.push((fqn, const_type));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Recurse through control-flow wrappers (`if`/`while`/`for`/`foreach`/
            // `do`/`switch`/`try`/blocks) so a declaration nested inside one is
            // collected identically to a top-level declaration. This is what makes
            // Laravel's global helpers visible: every one is declared inside an
            // `if (! function_exists('foo')) { function foo() {} }` guard, as are
            // Symfony polyfills and WordPress pluggable functions. Indexing is
            // unconditional — the guard is a runtime concern (which copy wins when
            // the file loads), irrelevant to static symbol candidacy, and the
            // codebase dedups by FQCN so a polyfill declared in several packages
            // produces no redeclaration noise.
            //
            // `walk_owned_stmt` only re-enters `visit_stmt` for nested statements;
            // it never reaches a declaration's own body because the `Function`/
            // `Class`/`Interface`/`Trait`/`Enum` arms above return without walking.
            // Closures and anonymous classes live in expressions, which `visit_expr`
            // (below) deliberately does not descend into — so a function declared
            // inside a closure is not wrongly registered at file scope.
            _ => return walk_owned_stmt(self, stmt),
        }
        ControlFlow::Continue(())
    }

    /// The collector registers statement-level declarations only; it has no
    /// reason to look inside expressions. Overriding this to a no-op stops the
    /// control-flow recursion in `visit_stmt` from descending into closure and
    /// arrow-function bodies (reached via `walk_owned_stmt`'s expression walk),
    /// which would otherwise register locally-scoped declarations at file scope.
    fn visit_expr(&mut self, _expr: &php_ast::owned::Expr) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

impl<'a> DefinitionCollector<'a> {
    fn build_method_storage(
        &mut self,
        m: &php_ast::owned::MethodDecl,
        class_fqcn: &str,
        span: Option<&php_ast::Span>,
        aliases: Option<&FxHashMap<String, Type>>,
        class_template_params: &[TemplateParam],
    ) -> Option<MethodDef> {
        let doc = m
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default();

        if let Some(c) = m.doc_comment.as_ref() {
            self.emit_docblock_issues(&doc, c.span.start);
        }

        if !self.version_allows(&doc) || !self.version_attr_available(&m.attributes) {
            return None;
        }

        // Merge method-level type aliases with the class-level ones. Method aliases
        // (defined via `@psalm-type` / `@phpstan-type` on the method docblock) take
        // precedence; class aliases fill in the rest.
        let method_type_aliases = self.build_type_aliases(&doc);
        let merged_aliases: FxHashMap<String, mir_types::Type> = if method_type_aliases.is_empty() {
            aliases.cloned().unwrap_or_default()
        } else {
            let mut merged = aliases.cloned().unwrap_or_default();
            merged.extend(method_type_aliases);
            merged
        };
        let effective_aliases: Option<&FxHashMap<String, mir_types::Type>> =
            if merged_aliases.is_empty() {
                None
            } else {
                Some(&merged_aliases)
            };

        // Build combined template name set before param resolution so docblock param types
        // that reference class-level template params (e.g. `TRelatedModel`) are stored as
        // TTemplateParam instead of being wrongly namespace-qualified.
        // Includes both method-level and class-level template names.
        let template_names: rustc_hash::FxHashSet<String> = doc
            .templates
            .iter()
            .map(|(n, _, _, _)| n.to_string())
            .chain(
                class_template_params
                    .iter()
                    .map(|tp| tp.name.as_ref().to_string()),
            )
            .collect();

        // Extract template params; bounds are resolved with template-awareness so a bound
        // that is itself a template param (e.g. `@template T of A` where A is another
        // template) is stored as TTemplateParam rather than being wrongly FQN-qualified.
        let template_params: Vec<TemplateParam> = doc
            .templates
            .iter()
            .map(|(name, bound, variance, default)| TemplateParam {
                name: name.as_str().into(),
                bound: wrap_template_bound(bound.clone().map(|b| {
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            b,
                            &template_names,
                            class_fqcn,
                            class_template_params,
                        ),
                        class_fqcn,
                    )
                })),
                default: wrap_template_bound(default.clone().map(|d| {
                    Self::fill_self_static_parent(
                        self.resolve_union_doc_with_templates(
                            d,
                            &template_names,
                            class_fqcn,
                            class_template_params,
                        ),
                        class_fqcn,
                    )
                })),
                defining_entity: class_fqcn.into(),
                variance: *variance,
            })
            .collect();

        // Combined param list for bound lookup: method-level first (they shadow class-level),
        // then class-level. Used only for resolve_union_doc_with_templates, not stored in MethodDef.
        let combined_template_params: Vec<TemplateParam>;
        let template_params_for_resolve: &[TemplateParam] = if class_template_params.is_empty() {
            &template_params
        } else {
            combined_template_params = template_params
                .iter()
                .chain(
                    class_template_params
                        .iter()
                        .filter(|ctp| !template_params.iter().any(|tp| tp.name == ctp.name)),
                )
                .cloned()
                .collect();
            &combined_template_params
        };

        let mut params = Vec::new();
        let mut local_scalar = 0usize;
        let mut local_complex = 0usize;
        let mut local_defaults = 0usize;
        for p in m.params.iter() {
            // phpstorm-stubs `#[PhpStormStubsElementAvailable]`: omit a param
            // that does not exist at the target version (preserves arity).
            if !self.version_attr_available(&p.attributes) {
                continue;
            }
            let param_name = p.name.as_deref().unwrap_or_default();
            let native_ty = self.resolve_union_opt(
                p.type_hint
                    .as_ref()
                    .map(|h| type_from_hint_owned(h, Some(class_fqcn))),
            );
            let ty = self
                // phpstorm-stubs `#[LanguageLevelTypeAware]` type override wins.
                .version_attr_type_string(&p.attributes)
                .map(|s| crate::parser::docblock::parse_type_string(&s))
                .or_else(|| {
                    doc.get_param_type(param_name).cloned().map(|u| {
                        // Use full_qualify=false resolution (same as before) so bare
                        // names like `Closure` stay bare and don't get namespaced.
                        // After that, run a template-only substitution pass to convert
                        // bare names matching class/method template params (e.g. TRelatedModel)
                        // into TTemplateParam without touching other names.
                        let resolved = effective_aliases
                            .map(|a| self.resolve_union_doc_with_aliases(u.clone(), a))
                            .unwrap_or_else(|| self.resolve_union_doc(u));
                        let doc_ty = self.substitute_template_params(
                            resolved,
                            &template_names,
                            template_params_for_resolve,
                            class_fqcn,
                        );
                        // When the native hint is a concrete scalar and the docblock has only
                        // atoms from a different scalar family (e.g. `@param int` + `bool` hint),
                        // the PHP type hint is the runtime truth — prefer it over the docblock.
                        if native_ty
                            .as_ref()
                            .is_some_and(|n| native_hint_wins_over_docblock_scalar(n, &doc_ty))
                        {
                            return native_ty.clone().unwrap();
                        }
                        // Partial conflict (e.g. `@param int|string` on a native `int`
                        // hint): strip the atoms foreign to the hint's family instead
                        // of storing the raw union.
                        let mut doc_ty = match native_ty.as_ref() {
                            Some(n) => resolve_docblock_scalar_conflict(n, doc_ty),
                            None => doc_ty,
                        };
                        // Mark the type as docblock-sourced so signature checks (e.g.
                        // param contravariance) can tell a `@param` refinement apart
                        // from a native type hint.
                        doc_ty.from_docblock = true;
                        doc_ty
                    })
                })
                .or(native_ty);
            if let Some(ty_ref) = &ty {
                if is_simple_scalar(ty_ref) {
                    local_scalar += 1;
                } else {
                    local_complex += 1;
                }
            }
            let has_default = p.default.is_some();
            if has_default {
                local_defaults += 1;
            }

            let out_ty = doc.get_out_param_type(param_name).cloned().map(|u| {
                let resolved = effective_aliases
                    .map(|a| self.resolve_union_doc_with_aliases(u.clone(), a))
                    .unwrap_or_else(|| self.resolve_union_doc(u));
                let mut resolved = self.substitute_template_params(
                    resolved,
                    &template_names,
                    template_params_for_resolve,
                    class_fqcn,
                );
                resolved.from_docblock = true;
                resolved
            });
            params.push(DeclaredParam {
                name: Name::new(param_name),
                ty: mir_codebase::wrap_param_type(ty),
                out_ty: mir_codebase::wrap_param_type(out_ty),
                has_default,
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: has_default || p.variadic,
            });
        }
        if local_scalar > 0 {
            SCALAR_PARAM_COUNT.fetch_add(local_scalar, Relaxed);
        }
        if local_complex > 0 {
            COMPLEX_PARAM_COUNT.fetch_add(local_complex, Relaxed);
        }
        if local_defaults > 0 {
            PARAM_WITH_DEFAULT.fetch_add(local_defaults, Relaxed);
        }

        // Same func_get_args detection as for free functions (see collector/function.rs).
        let last_is_variadic = params.last().is_some_and(|p| p.is_variadic);
        if !last_is_variadic {
            let body_stmts = m
                .body
                .as_deref()
                .map(|b| b.stmts.as_ref())
                .unwrap_or_default();
            if crate::collector::function::stmts_use_func_get_args(body_stmts) {
                params.push(DeclaredParam {
                    name: mir_types::Name::new("..."),
                    ty: None,
                    out_ty: None,
                    has_default: false,
                    is_variadic: true,
                    is_byref: false,
                    is_optional: true,
                });
            }
        }

        // phpstorm-stubs `#[LanguageLevelTypeAware]` return type wins, routed
        // through the same resolution + self/static/parent filling.
        let attr_return = self.version_attr_type_string(&m.attributes).map(|s| {
            let mut ty = crate::parser::docblock::parse_type_string(&s);
            ty.from_docblock = true;
            ty
        });
        let return_type = match (
            attr_return.or_else(|| doc.return_type.clone()),
            m.return_type.as_ref(),
        ) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                // Expand type aliases first (no FQN change), then resolve.
                let expanded = effective_aliases
                    .map_or(ty.clone(), |a| self.expand_aliases_only(ty.clone(), a));
                let resolved = if !template_names.is_empty() {
                    // Use template-aware resolution: FQN-qualifies the outer class in
                    // generic return types (e.g. ObjectProphecy<T>) and converts T to
                    // TTemplateParam.
                    self.resolve_union_doc_with_templates(
                        expanded,
                        &template_names,
                        class_fqcn,
                        template_params_for_resolve,
                    )
                } else {
                    self.resolve_union_doc(expanded)
                };
                Some(Self::fill_self_static_parent(resolved, class_fqcn))
            }
            (None, Some(h)) => {
                self.resolve_union_opt(Some(type_from_hint_owned(h, Some(class_fqcn))))
            }
            (None, None) => None,
        };

        let throws = doc
            .throws
            .iter()
            .map(|t| {
                Arc::from(resolution::resolve_name(t, &self.namespace, &self.use_aliases).as_str())
            })
            .collect();

        // Resolve `@if-this-is` while the template-param borrow is still live
        // (it must not outlive the `template_params` move into MethodDef below).
        let if_this_is_resolved: Option<Arc<Type>> = doc.if_this_is.clone().map(|mut ty| {
            ty.from_docblock = true;
            let resolved = if template_names.is_empty() {
                self.resolve_union_doc(ty)
            } else {
                self.resolve_union_doc_with_templates(
                    ty,
                    &template_names,
                    class_fqcn,
                    template_params_for_resolve,
                )
            };
            Arc::new(Self::fill_self_static_parent(resolved, class_fqcn))
        });

        // Resolve `@psalm-self-out` the same way as `@if-this-is` above.
        let self_out_resolved: Option<Arc<Type>> = doc.self_out.clone().map(|mut ty| {
            ty.from_docblock = true;
            let resolved = if template_names.is_empty() {
                self.resolve_union_doc(ty)
            } else {
                self.resolve_union_doc_with_templates(
                    ty,
                    &template_names,
                    class_fqcn,
                    template_params_for_resolve,
                )
            };
            Arc::new(Self::fill_self_static_parent(resolved, class_fqcn))
        });

        let method_name = m.name.as_deref().unwrap_or_default();
        let is_override = m.attributes.iter().any(|a| {
            a.name
                .parts
                .last()
                .map(|p| p.as_ref().eq_ignore_ascii_case("Override"))
                .unwrap_or(false)
        });
        Some(MethodDef {
            name: Arc::from(method_name),
            fqcn: class_fqcn.into(),
            params: Arc::from(params.into_boxed_slice()),
            return_type: wrap_return_type(return_type),
            inferred_return_type: None,
            visibility: Self::convert_visibility(m.visibility),
            is_static: m.is_static,
            is_abstract: m.is_abstract,
            is_final: m.is_final,
            is_constructor: method_name == "__construct",
            template_params,
            assertions: self.build_assertions(&doc),
            throws,
            deprecated: doc.deprecated.as_deref().map(Arc::from).or_else(|| {
                if m.attributes.iter().any(|a| {
                    a.name
                        .parts
                        .last()
                        .map(|p| p.as_ref().eq_ignore_ascii_case("Deprecated"))
                        .unwrap_or(false)
                }) {
                    Some(Arc::from(""))
                } else {
                    None
                }
            }),
            is_internal: doc.is_internal,
            is_pure: doc.is_pure,
            no_named_arguments: doc.no_named_arguments,
            is_override,
            is_virtual: false,
            location: span.map(|s| self.location(s.start, s.end)),
            docstring: if doc.description.trim().is_empty() {
                None
            } else {
                Some(Arc::from(doc.description.as_str()))
            },
            taint_sink_params: doc
                .taint_sinks
                .iter()
                .map(|(param, kind)| (Arc::from(param.as_str()), Arc::from(kind.as_str())))
                .collect(),
            if_this_is: if_this_is_resolved,
            self_out: self_out_resolved,
            is_inherit_doc: doc.is_inherit_doc,
            is_mutation_free: doc.is_mutation_free,
            is_external_mutation_free: doc.is_external_mutation_free,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_and_collect_slice(file: &str, src: &str) -> StubSlice {
        let result = php_rs_parser::parse(src);
        let collector =
            DefinitionCollector::new_for_slice(Arc::from(file), src, &result.source_map);
        let (slice, _) = collector.collect_slice(&result.program);
        slice
    }

    // These three tests guard the DefinitionCollector → StubSlice contract for
    // namespace and import data.
    //
    // Background: collect_slice is the pure output path used by incremental /
    // salsa pipelines (LSP, re_analyze_file). For StubSlice-based consumers to
    // produce correct diagnostics, the slice must carry the same namespace and
    // import data that project.rs collects via its separate AST walk. If either
    // field is missing from the slice, StatementsAnalyzer receives empty maps
    // during body analysis and emits false UndefinedClass diagnostics for use-aliased
    // or same-namespace classes.

    #[test]
    fn collect_slice_captures_namespace() {
        // The first namespace declaration must end up in slice.namespace so
        // that file_namespace() can derive it via collect_file_definitions.
        let slice = parse_and_collect_slice(
            "src/Service.php",
            "<?php\nnamespace App\\Service;\nclass Handler {}\n",
        );
        assert_eq!(
            slice.namespace.as_deref(),
            Some("App\\Service"),
            "collect_slice must capture the file namespace"
        );
    }

    #[test]
    fn collect_slice_captures_use_imports() {
        // All `use` imports (plain and aliased) must end up in slice.imports so
        // that file_imports() can derive them via collect_file_definitions and
        // body analysis can resolve short names like `new Entity()` correctly.
        let slice = parse_and_collect_slice(
            "src/Handler.php",
            "<?php\nnamespace App\\Service;\nuse App\\Model\\Entity;\nuse App\\Repository\\EntityRepo as Repo;\nclass Handler {}\n",
        );
        let imports = &slice.imports;
        assert_eq!(
            imports
                .get(&mir_types::Name::new("Entity"))
                .map(|s| s.as_str()),
            Some("App\\Model\\Entity"),
            "collect_slice must capture plain use import"
        );
        assert_eq!(
            imports
                .get(&mir_types::Name::new("Repo"))
                .map(|s| s.as_str()),
            Some("App\\Repository\\EntityRepo"),
            "collect_slice must capture aliased use import"
        );
    }

    #[test]
    fn collect_slice_class_imports_excludes_use_function_and_const() {
        // `use function`/`use const` (plain or grouped) must still land in
        // slice.imports (Pass-2 function-call resolution needs them), but never
        // in slice.class_imports — otherwise a same-named class/type-hint
        // reference would incorrectly resolve to the function/constant's FQN.
        let slice = parse_and_collect_slice(
            "src/Handler.php",
            concat!(
                "<?php\n",
                "namespace App\\Service;\n",
                "use App\\Model\\Entity;\n",
                "use function App\\Helpers\\foo;\n",
                "use const App\\Helpers\\BAR;\n",
                "use App\\Helpers\\{Baz, function qux, const QUUX};\n",
                "class Handler {}\n",
            ),
        );

        for (alias, fqcn) in [
            ("Entity", "App\\Model\\Entity"),
            ("foo", "App\\Helpers\\foo"),
            ("BAR", "App\\Helpers\\BAR"),
            ("Baz", "App\\Helpers\\Baz"),
            ("qux", "App\\Helpers\\qux"),
            ("QUUX", "App\\Helpers\\QUUX"),
        ] {
            assert_eq!(
                slice.imports.get(&mir_types::Name::new(alias)).map(|s| s.as_str()),
                Some(fqcn),
                "slice.imports must capture every UseKind for alias {alias}"
            );
        }

        assert_eq!(
            slice
                .class_imports
                .get(&mir_types::Name::new("Entity"))
                .map(|s| s.as_str()),
            Some("App\\Model\\Entity"),
            "slice.class_imports must capture a Normal-kind alias"
        );
        assert_eq!(
            slice
                .class_imports
                .get(&mir_types::Name::new("Baz"))
                .map(|s| s.as_str()),
            Some("App\\Helpers\\Baz"),
            "slice.class_imports must capture a Normal-kind alias from a grouped use"
        );
        for alias in ["foo", "BAR", "qux", "QUUX"] {
            assert!(
                slice.class_imports.get(&mir_types::Name::new(alias)).is_none(),
                "slice.class_imports must not contain function/const alias {alias}"
            );
        }
    }

    #[test]
    fn collect_slice_captures_namespace_none_when_no_namespace() {
        // Global-scope files have no namespace declaration; slice.namespace must
        // be None so file_namespace() correctly returns None for global-scope files.
        let slice = parse_and_collect_slice("src/global.php", "<?php\nfunction foo(): void {}\n");
        assert!(
            slice.namespace.is_none(),
            "collect_slice must not set namespace for global-scope files"
        );
    }

    #[test]
    fn trait_require_extends_is_collected() {
        let src = r#"<?php
class Model {}

/**
 * @psalm-require-extends Model
 */
trait HasTimestamps {}
"#;
        let slice = parse_and_collect_slice("test.php", src);
        let tr = slice
            .traits
            .iter()
            .find(|tr| tr.fqcn.as_ref() == "HasTimestamps")
            .expect("HasTimestamps should be collected");
        assert_eq!(
            tr.require_extends,
            vec![std::sync::Arc::from("Model")],
            "require_extends should contain Model"
        );
    }

    #[test]
    fn trait_require_extends_via_project_analyzer() {
        let src = r#"<?php
/** @psalm-require-extends Model */
trait HasTimestamps {
    public function touch(): void {}
}

class Model {}

class NotAModel {
    use HasTimestamps;
}
"#;
        let result = crate::test_utils::check(src);
        assert!(
            result.iter().any(|i| i.kind.name() == "InvalidTraitUse"),
            "Expected InvalidTraitUse issue"
        );
    }
}
