use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
/// definition collection — Definition collector.
///
/// Visits every top-level declaration in the AST and produces a `StubSlice`
/// containing class, function, and constant signatures. No type inference
/// happens here.
use std::sync::Arc;

use std::ops::ControlFlow;

use php_ast::ast::Visibility as AstVisibility;
use php_ast::owned::visitor::{walk_owned_program, OwnedVisitor};
use php_ast::owned::{Program, StmtKind};

use crate::parser::{name_to_string_owned, type_from_hint_owned};
use crate::php_version::PhpVersion;
use mir_codebase::storage::{
    wrap_return_type, Assertion, FnParam, MethodDef, PropertyDef, StubSlice, TemplateParam,
    Visibility,
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
            | Atomic::TBool
            | Atomic::TMixed
            | Atomic::TNull
            | Atomic::TVoid
            | Atomic::TNever
    )
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
fn infer_const_value(expr_kind: &php_ast::owned::ExprKind) -> Option<Type> {
    use php_ast::ast::UnaryPrefixOp;

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
    /// `use` aliases: alias → FQCN
    use_aliases: FxHashMap<String, String>,
    issues: IssueBuffer,
    /// When `Some`, stub symbols annotated with `@since`/`@removed` are filtered
    /// against this target version. `None` disables filtering (user code).
    php_version: Option<PhpVersion>,
    /// The first namespace declaration seen in this file. Matches the semantics
    /// of `project.rs` which only records the first namespace per file.
    first_namespace: Option<String>,
    /// All `use` imports ever encountered in this file, accumulated across all
    /// namespace blocks. Unlike `use_aliases`, this is never cleared or restored,
    /// so braced-namespace imports are not lost.
    accumulated_imports: FxHashMap<String, String>,
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

    /// Parse a docblock from a node's doc_comment, falling back to preceding docblock if not found.
    fn parse_docblock_from_node_or_preceding(
        &self,
        doc_comment: Option<&php_ast::owned::Comment>,
        span_start: u32,
    ) -> crate::parser::ParsedDocblock {
        doc_comment
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .or_else(|| {
                crate::parser::find_preceding_docblock(self.source, span_start)
                    .map(|t| crate::parser::DocblockParser::parse(&t))
            })
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
    }

    pub fn collect_slice(mut self, program: &Program) -> (StubSlice, Vec<Issue>) {
        let _ = self.visit_program(program);
        self.finalize_slice();
        (self.slice, self.issues.into_issues())
    }

    // -----------------------------------------------------------------------
    // FQCN resolution helpers
    // -----------------------------------------------------------------------
    // Type Resolution (delegating to resolution module)
    // -----------------------------------------------------------------------

    fn resolve_name(&self, name: &str) -> String {
        resolution::resolve_name(name, &self.namespace, &self.use_aliases)
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
        template_names: &std::collections::HashSet<String>,
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
                        .and_then(|tp| tp.bound.clone())
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
        own_methods: &mut indexmap::IndexMap<Arc<str>, Arc<MethodDef>>,
        own_properties: &mut indexmap::IndexMap<Arc<str>, PropertyDef>,
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
                    ty,
                    inferred_ty: None,
                    visibility: Visibility::Public,
                    is_static: false,
                    is_readonly: prop.read_only,
                    default: None,
                    location: location.clone(),
                },
            );
        }

        for method in &doc.methods {
            if method.name.is_empty() {
                continue;
            }
            let key = Arc::from(method.name.to_lowercase().as_str());
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
                    FnParam {
                        name: Name::new(p.name.as_str()),
                        ty: mir_codebase::wrap_param_type(ty),
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
                    location: location.clone(),
                    docstring: None,
                    is_virtual: true,
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
        let Some(doc_text) = crate::parser::find_preceding_docblock(self.source, stmt.span.start)
        else {
            return;
        };
        let parsed = crate::parser::DocblockParser::parse(&doc_text);
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
                        let flow = self.process_stmts(stmts);
                        self.use_aliases = saved_aliases;
                        flow?;
                    }
                    php_ast::owned::NamespaceBody::Simple => {
                        // Simple namespace — affects all subsequent declarations
                    }
                }
            }

            StmtKind::Use(use_decl) => {
                for item in use_decl.uses.iter() {
                    let full_name = name_to_string_owned(&item.name)
                        .trim_start_matches('\\')
                        .to_string();
                    let alias = item
                        .alias
                        .as_deref()
                        .unwrap_or_else(|| full_name.rsplit('\\').next().unwrap_or(&full_name));
                    self.use_aliases
                        .insert(alias.to_string(), full_name.clone());
                    self.accumulated_imports
                        .insert(alias.to_string(), full_name);
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
                        .or_else(|| {
                            crate::parser::find_preceding_docblock(self.source, item.span.start)
                                .map(|t| crate::parser::DocblockParser::parse(&t))
                        })
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

            StmtKind::Block(stmts) => {
                return self.process_stmts(stmts);
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
                                    // Check for @since/@removed on the docblock preceding this define().
                                    let define_doc = crate::parser::find_preceding_docblock(
                                        self.source,
                                        stmt.span.start,
                                    )
                                    .map(|t| crate::parser::DocblockParser::parse(&t))
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

            _ => {}
        }
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

        if !self.version_allows(&doc) {
            return None;
        }

        let mut params = Vec::new();
        let mut local_scalar = 0usize;
        let mut local_complex = 0usize;
        let mut local_defaults = 0usize;
        for p in m.params.iter() {
            let param_name = p.name.as_deref().unwrap_or_default();
            let ty = doc
                .get_param_type(param_name)
                .cloned()
                .map(|u| {
                    aliases
                        .map(|a| self.resolve_union_doc_with_aliases(u.clone(), a))
                        .unwrap_or_else(|| self.resolve_union_doc(u))
                })
                .or_else(|| {
                    self.resolve_union_opt(
                        p.type_hint
                            .as_ref()
                            .map(|h| type_from_hint_owned(h, Some(class_fqcn))),
                    )
                });
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

            params.push(FnParam {
                name: Name::new(param_name),
                ty: mir_codebase::wrap_param_type(ty),
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

        // Extract template params before processing return type so generic return types
        // like ObjectProphecy<T> can be resolved with template-awareness (FQN-qualifying
        // the outer class while converting T to TTemplateParam).
        let template_params: Vec<TemplateParam> = doc
            .templates
            .iter()
            .map(|(name, bound, variance)| TemplateParam {
                name: name.as_str().into(),
                bound: self.resolve_union_opt(bound.clone()),
                defining_entity: class_fqcn.into(),
                variance: *variance,
            })
            .collect();

        // Build combined template name set: method-level templates union class-level templates.
        // This prevents class-level template params (e.g. TKey on a generic class) from being
        // treated as bare class names and wrongly namespace-qualified by the TNamedObject branch.
        let template_names: std::collections::HashSet<String> = doc
            .templates
            .iter()
            .map(|(n, _, _)| n.to_string())
            .chain(
                class_template_params
                    .iter()
                    .map(|tp| tp.name.as_ref().to_string()),
            )
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

        let return_type = match (doc.return_type.clone(), m.return_type.as_ref()) {
            (Some(mut ty), _) => {
                ty.from_docblock = true;
                let resolved = if !template_names.is_empty() {
                    // Use template-aware resolution: FQN-qualifies the outer class in
                    // generic return types (e.g. ObjectProphecy<T>) and converts T to
                    // TTemplateParam.
                    self.resolve_union_doc_with_templates(
                        ty,
                        &template_names,
                        class_fqcn,
                        template_params_for_resolve,
                    )
                } else {
                    aliases
                        .map(|a| self.resolve_union_doc_with_aliases(ty.clone(), a))
                        .unwrap_or_else(|| self.resolve_union_doc(ty))
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

        let method_name = m.name.as_deref().unwrap_or_default();
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
            deprecated: doc.deprecated.as_deref().map(Arc::from),
            is_internal: doc.is_internal,
            is_pure: doc.is_pure,
            is_virtual: false,
            location: span.map(|s| self.location(s.start, s.end)),
            docstring: if doc.description.trim().is_empty() {
                None
            } else {
                Some(Arc::from(doc.description.as_str()))
            },
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
