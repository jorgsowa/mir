use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_issues::Issue;
use mir_types::{Name, Type};
use parking_lot::Mutex;

use crate::db::{resolve_name, MirDatabase};
use crate::diagnostics::{
    check_expr_for_undefined_classes, check_name_class, check_type_hint_classes,
    collect_type_hint_class_refs, emit_unused_params, emit_unused_variables,
};
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

/// Calls `f` on every file-scope statement that is **not** a control-flow
/// wrapper, recursing into the bodies of `if`/`elseif`/`else`, `while`, `for`,
/// `foreach`, `do`/`while`, `switch`, `try`/`catch`/`finally`, bare blocks, and
/// braced namespaces to any depth. It never descends into a declaration's own
/// body, into closures, or into expressions — so callers see exactly the
/// declarations and other simple statements that live at file scope, whether or
/// not they are wrapped in conditional guards.
///
/// This exists because conditionally-declared symbols must be discoverable
/// identically to top-level ones: Laravel declares every global helper inside an
/// `if (! function_exists('foo')) { function foo() {} }` guard (as do Symfony
/// polyfills and WordPress pluggable functions). It mirrors the statement
/// recursion in `php_ast`'s `walk_owned_stmt`, kept as a standalone
/// borrow-transparent walk because the callers are not `OwnedVisitor`s and one
/// of them ([`crate::db::per_function`]) must return a reference tied to the
/// program's lifetime, which the visitor trait's elided lifetimes cannot express.
pub(crate) fn for_each_file_scope_decl<'a>(
    stmts: &'a [php_ast::owned::Stmt],
    f: &mut dyn FnMut(&'a php_ast::owned::Stmt),
) {
    for stmt in stmts.iter() {
        visit_file_scope_stmt(stmt, f);
    }
}

fn visit_file_scope_stmt<'a>(
    stmt: &'a php_ast::owned::Stmt,
    f: &mut dyn FnMut(&'a php_ast::owned::Stmt),
) {
    use php_ast::owned::{NamespaceBody, StmtKind};
    match &stmt.kind {
        StmtKind::If(s) => {
            visit_file_scope_stmt(&s.then_branch, f);
            for branch in s.elseif_branches.iter() {
                visit_file_scope_stmt(&branch.body, f);
            }
            if let Some(else_branch) = &s.else_branch {
                visit_file_scope_stmt(else_branch, f);
            }
        }
        StmtKind::While(s) => visit_file_scope_stmt(&s.body, f),
        StmtKind::For(s) => visit_file_scope_stmt(&s.body, f),
        StmtKind::Foreach(s) => visit_file_scope_stmt(&s.body, f),
        StmtKind::DoWhile(s) => visit_file_scope_stmt(&s.body, f),
        StmtKind::Switch(s) => {
            for case in s.body.cases.iter() {
                for inner in case.body.iter() {
                    visit_file_scope_stmt(inner, f);
                }
            }
        }
        StmtKind::TryCatch(t) => {
            for inner in t.body.stmts.iter() {
                visit_file_scope_stmt(inner, f);
            }
            for catch in t.catches.iter() {
                for inner in catch.body.stmts.iter() {
                    visit_file_scope_stmt(inner, f);
                }
            }
            if let Some(finally) = &t.finally {
                for inner in finally.stmts.iter() {
                    visit_file_scope_stmt(inner, f);
                }
            }
        }
        StmtKind::Block(b) => {
            for inner in b.stmts.iter() {
                visit_file_scope_stmt(inner, f);
            }
        }
        StmtKind::Declare(d) => {
            // Block form: `declare(strict_types=1) { function foo() {} }`.
            // Kept in lockstep with the collector's `walk_owned_stmt`, which also
            // descends into declare bodies, so both walks agree on what counts as
            // a file-scope declaration.
            if let Some(body) = &d.body {
                visit_file_scope_stmt(body, f);
            }
        }
        StmtKind::Namespace(ns) => {
            if let NamespaceBody::Braced(block) = &ns.body {
                for inner in block.stmts.iter() {
                    visit_file_scope_stmt(inner, f);
                }
            }
        }
        _ => f(stmt),
    }
}

/// Controls which side-effects the analysis passes perform.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum AnalysisMode {
    /// Full analysis: emits diagnostics, records reference locations, and
    /// tracks symbol definitions.
    Full,
    /// Inference-only (priming) pass: collects inferred function/method return
    /// types into the shared store. Skips reference location recording,
    /// symbol tracking, and top-level diagnostic emission so those locations
    /// are not double-counted by the subsequent `Full` pass.
    InferenceOnly,
}

#[derive(Clone)]
pub(crate) struct InferredTypes {
    pub(crate) functions: Vec<(Arc<str>, Type)>,
    pub(crate) methods: Vec<(Arc<str>, Arc<str>, Type)>,
}

/// Look up `(params, return_ty, template_params, throws)` for a method via
/// the inheritance chain. Empty defaults when nothing resolves.
#[allow(clippy::type_complexity)]
fn method_chain_signature(
    db: &dyn MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> (
    Arc<[mir_codebase::storage::FnParam]>,
    Option<Type>,
    Vec<mir_codebase::storage::TemplateParam>,
    Arc<[Arc<str>]>,
) {
    let method_name_lower = if method_name.bytes().any(|b| b.is_ascii_uppercase()) {
        std::borrow::Cow::Owned(method_name.to_ascii_lowercase())
    } else {
        std::borrow::Cow::Borrowed(method_name)
    };
    if let Some((_, storage)) = crate::db::find_method_in_chain(
        db,
        crate::db::Fqcn::from_str(db, fqcn),
        method_name_lower.as_ref(),
    ) {
        let fqcn_interned = crate::db::Fqcn::from_str(db, fqcn);
        let parent = crate::db::find_inheritdoc_parent(
            db,
            fqcn_interned,
            fqcn_interned,
            method_name_lower.as_ref(),
            &storage,
        );

        let own_has_docblock_return = storage
            .return_type
            .as_deref()
            .map(|t| t.from_docblock)
            .unwrap_or(false);
        let return_type = if own_has_docblock_return {
            storage.return_type.as_deref().cloned()
        } else {
            parent
                .as_ref()
                .and_then(|p| p.return_type.as_deref().cloned())
                .or_else(|| storage.return_type.as_deref().cloned())
        };

        let params: Arc<[mir_codebase::storage::FnParam]> = if let Some(ref p) = parent {
            storage
                .params
                .iter()
                .enumerate()
                .map(|(i, own)| {
                    let own_ty_is_docblock =
                        own.ty.as_deref().map(|t| t.from_docblock).unwrap_or(false);
                    let own_is_mixed_or_absent =
                        own.ty.as_deref().map(|t| t.is_mixed()).unwrap_or(true);
                    if !own_ty_is_docblock && own_is_mixed_or_absent {
                        if let Some(parent_param) = p.params.get(i) {
                            if parent_param.ty.is_some() {
                                return mir_codebase::storage::FnParam {
                                    ty: parent_param.ty.clone(),
                                    ..own.clone()
                                };
                            }
                        }
                    }
                    own.clone()
                })
                .collect::<Vec<_>>()
                .into()
        } else {
            Arc::clone(&storage.params)
        };

        let template_params = if storage.template_params.is_empty() {
            parent
                .as_ref()
                .map(|p| p.template_params.clone())
                .unwrap_or_default()
        } else {
            storage.template_params.clone()
        };

        let throws: Arc<[Arc<str>]> = if storage.throws.is_empty() {
            parent
                .as_ref()
                .map(|p| Arc::from(p.throws.as_slice()))
                .unwrap_or_else(|| Arc::from([]))
        } else {
            Arc::from(storage.throws.as_slice())
        };

        return (params, return_type, template_params, throws);
    }
    (Arc::from([]), None, vec![], Arc::from([]))
}

/// Resolve a function declaration's storage via the salsa pull path
/// (qualified FQN → raw name → short-name scan over `workspace_functions`).
fn lookup_function_node_for_decl(
    db: &dyn MirDatabase,
    file: &str,
    fn_name: &str,
) -> Option<(Arc<str>, Arc<mir_codebase::storage::FunctionDef>)> {
    let qualified = resolve_name(db, file, fn_name);
    let try_lookup = |fqn: &str| -> Option<Arc<mir_codebase::storage::FunctionDef>> {
        crate::db::find_function(db, crate::db::Fqcn::from_str(db, fqn))
    };
    if let Some(f) = try_lookup(qualified.as_str()) {
        return Some((Arc::from(qualified), f));
    }
    if let Some(f) = try_lookup(fn_name) {
        return Some((Arc::from(fn_name), f));
    }
    crate::metrics::record_fn_short_name_scan();
    for fqn in crate::db::workspace_functions(db).iter() {
        let short = fqn.rsplit('\\').next().unwrap_or(fqn.as_ref());
        if short == fn_name {
            if let Some(f) = try_lookup(fqn.as_ref()) {
                return Some((fqn.clone(), f));
            }
        }
    }
    None
}

/// Build `FnParam`s directly from the declaration AST when no storage match is
/// available.  Defaults are typed as `mixed` since their value type isn't tracked.
fn ast_derived_fn_params(params: &[php_ast::owned::Param]) -> Vec<mir_codebase::FnParam> {
    params
        .iter()
        .map(|p| mir_codebase::FnParam {
            name: Name::new(p.name.as_deref().unwrap_or("")),
            ty: None,
            has_default: p.default.is_some(),
            is_variadic: p.variadic,
            is_byref: p.by_ref,
            is_optional: p.default.is_some() || p.variadic,
        })
        .collect()
}

/// Walk top-level statements (recursing into braced namespaces) and collect
/// Kind of a top-level symbol declaration, used for duplicate detection.
#[derive(Clone, Copy)]
enum DeclKind {
    Class,
    Interface,
    Trait,
    Enum,
    Function,
}

/// Collect the fully-qualified name, span, and kind of every top-level
/// declaration in `stmts`. The `ns_prefix` tracks the current resolved
/// namespace (e.g. `"Aye\\"` or `""`).
///
/// Classes, interfaces, traits, and enums share one PHP symbol namespace;
/// functions occupy a separate namespace. Both are collected into the same
/// output vec so the caller can build the appropriate seen-maps.
fn collect_decl_spans(
    stmts: &[php_ast::owned::Stmt],
    ns_prefix: &str,
    out: &mut Vec<(String, php_ast::Span, DeclKind)>,
) {
    use php_ast::owned::{NamespaceBody, StmtKind};
    // Track the prefix of the current unbraced (`namespace A;`) namespace as we
    // walk siblings. A file may switch namespaces several times, e.g.
    // `namespace A; class Foo {} namespace B; class Foo {}` declares the distinct
    // `A\Foo` and `B\Foo`, so each symbol must use the prefix in effect at its position.
    let mut current_prefix = ns_prefix.to_string();
    for stmt in stmts {
        match &stmt.kind {
            StmtKind::Class(d) => {
                let name = d.name.as_ref().and_then(|n| n.as_deref()).unwrap_or("");
                out.push((
                    format!("{current_prefix}{name}"),
                    stmt.span,
                    DeclKind::Class,
                ));
            }
            StmtKind::Interface(d) => {
                let name = d.name.as_deref().unwrap_or("");
                out.push((
                    format!("{current_prefix}{name}"),
                    stmt.span,
                    DeclKind::Interface,
                ));
            }
            StmtKind::Trait(d) => {
                let name = d.name.as_deref().unwrap_or("");
                out.push((
                    format!("{current_prefix}{name}"),
                    stmt.span,
                    DeclKind::Trait,
                ));
            }
            StmtKind::Enum(d) => {
                let name = d.name.as_deref().unwrap_or("");
                out.push((format!("{current_prefix}{name}"), stmt.span, DeclKind::Enum));
            }
            StmtKind::Function(d) => {
                let name = d.name.as_deref().unwrap_or("");
                out.push((
                    format!("{current_prefix}{name}"),
                    stmt.span,
                    DeclKind::Function,
                ));
            }
            StmtKind::Namespace(ns) => {
                let raw_ns = ns
                    .name
                    .as_ref()
                    .map(|n| {
                        n.parts
                            .iter()
                            .map(|p| p.as_ref())
                            .collect::<Vec<_>>()
                            .join("\\")
                    })
                    .unwrap_or_default();
                let child_prefix = if raw_ns.is_empty() {
                    String::new()
                } else {
                    format!("{raw_ns}\\")
                };
                match &ns.body {
                    NamespaceBody::Braced(block) => {
                        collect_decl_spans(&block.stmts, &child_prefix, out);
                    }
                    // Unbraced: the declarations that follow are flat siblings,
                    // so this prefix applies until the next `namespace` statement.
                    NamespaceBody::Simple => current_prefix = child_prefix,
                }
            }
            _ => {}
        }
    }
}

/// Emit `Duplicate*` issues for any symbol declared more than once in `stmts`.
///
/// Classes, interfaces, traits, and enums share one PHP symbol namespace
/// (a `class Foo` and an `interface Foo` in the same file is a fatal error).
/// Functions occupy their own namespace and are checked independently.
pub(crate) fn check_duplicate_declarations(
    stmts: &[php_ast::owned::Stmt],
    file: &Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    issues: &mut Vec<Issue>,
) {
    let mut decls: Vec<(String, php_ast::Span, DeclKind)> = Vec::new();
    collect_decl_spans(stmts, "", &mut decls);

    // Class-like namespace: class + interface + trait + enum all share one map.
    let mut seen_class_like: FxHashMap<Name, ()> = FxHashMap::default();
    // Function namespace is separate.
    let mut seen_fns: FxHashMap<Name, ()> = FxHashMap::default();

    for (fqn, span, kind) in &decls {
        let key = Name::from(fqn.as_str()).ascii_lowercase();
        let seen = match kind {
            DeclKind::Function => &mut seen_fns,
            _ => &mut seen_class_like,
        };
        if let std::collections::hash_map::Entry::Vacant(e) = seen.entry(key) {
            e.insert(());
        } else {
            // Emit on the second (and subsequent) occurrences.
            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, span.start, source_map);
            let (line_end, col_end) =
                crate::diagnostics::offset_to_line_col(source, span.end, source_map);
            let issue_kind = match kind {
                DeclKind::Class => mir_issues::IssueKind::DuplicateClass { name: fqn.clone() },
                DeclKind::Interface => {
                    mir_issues::IssueKind::DuplicateInterface { name: fqn.clone() }
                }
                DeclKind::Trait => mir_issues::IssueKind::DuplicateTrait { name: fqn.clone() },
                DeclKind::Enum => mir_issues::IssueKind::DuplicateEnum { name: fqn.clone() },
                DeclKind::Function => {
                    mir_issues::IssueKind::DuplicateFunction { name: fqn.clone() }
                }
            };
            issues.push(Issue::new(
                issue_kind,
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end,
                },
            ));
        }
    }
}

/// Container-kind parameters for one method-body analysis. Captures the
/// divergences between the class / trait / enum and plain / typed paths so
/// the shared [`BodyAnalyzer::analyze_method_scope`] core reproduces each
/// call site's behavior exactly:
///
/// - traits and enums have no parent class context (`parent_fqcn: None`);
/// - enums never treat `__construct` specially;
/// - missing-return and `__toString` checks run only on the untyped class
///   path (Full mode);
/// - docblock template params bind only on the untyped class path;
/// - parameter default-value expressions are analyzed only on class paths.
pub(crate) struct MethodScopeCx {
    pub fqcn: Arc<str>,
    pub parent_fqcn: Option<Arc<str>>,
    pub detect_ctor: bool,
    pub with_templates: bool,
    pub check_returns: bool,
    pub analyze_param_defaults: bool,
    pub strict_types: bool,
}

/// Returns `true` if `source` contains a top-level `declare(strict_types=1)`.
/// PHP mandates the directive appear before any other code, so scanning the
/// first 1 KiB is sufficient and reliable for well-formed files.
pub(crate) fn is_strict_types_file(source: &str) -> bool {
    let end = source.floor_char_boundary(source.len().min(1024));
    let prefix = &source[..end];
    prefix.contains("declare(strict_types=1)") || prefix.contains("declare(strict_types = 1)")
}

pub(crate) struct BodyAnalyzer<'a> {
    db: &'a dyn MirDatabase,
    php_version: PhpVersion,
    mode: AnalysisMode,
    inferred_types: Arc<Mutex<InferredTypes>>,
}

impl<'a> BodyAnalyzer<'a> {
    pub(crate) fn new(db: &'a dyn MirDatabase, php_version: PhpVersion) -> Self {
        Self {
            db,
            php_version,
            mode: AnalysisMode::Full,
            inferred_types: Arc::new(Mutex::new(InferredTypes {
                functions: Vec::new(),
                methods: Vec::new(),
            })),
        }
    }

    pub(crate) fn new_inference_only(db: &'a dyn MirDatabase, php_version: PhpVersion) -> Self {
        Self {
            db,
            php_version,
            mode: AnalysisMode::InferenceOnly,
            inferred_types: Arc::new(Mutex::new(InferredTypes {
                functions: Vec::new(),
                methods: Vec::new(),
            })),
        }
    }

    pub(crate) fn take_inferred_types(&self) -> InferredTypes {
        let types = Arc::clone(&self.inferred_types);
        Arc::try_unwrap(types)
            .map(|mutex| mutex.into_inner())
            .unwrap_or_else(|arc| arc.lock().clone())
    }

    fn record_function_inference(&self, fqn: &Arc<str>, inferred: &Type) {
        if self.mode == AnalysisMode::InferenceOnly {
            let mut types = self.inferred_types.lock();
            types.functions.push((fqn.clone(), inferred.clone()));
        }
    }

    fn record_method_inference(&self, fqcn: &str, name: &str, inferred: &Type) {
        if self.mode == AnalysisMode::InferenceOnly {
            let mut types = self.inferred_types.lock();
            types
                .methods
                .push((Arc::from(fqcn), Arc::from(name), inferred.clone()));
        }
    }

    fn check_and_record_type_hint_classes(
        &self,
        hint: &php_ast::owned::TypeHint,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        check_type_hint_classes(
            hint,
            self.db,
            file,
            source,
            source_map,
            all_issues,
            self.php_version,
        );
        if self.mode == AnalysisMode::Full {
            for (fqcn, span) in collect_type_hint_class_refs(hint, self.db, file) {
                let (line, col_start) =
                    crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                let (_, col_end) =
                    crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                self.db.record_reference_location(crate::db::RefLoc {
                    symbol_key: fqcn,
                    file: file.clone(),
                    line,
                    col_start,
                    col_end: col_end.max(col_start + 1),
                });
            }
        }
    }
}

mod aggregates;
mod classes;
mod functions;
mod orchestration;

/// Seed `ctx.var_locations` for function/method parameters using their AST spans.
fn seed_param_locations(
    ctx: &mut crate::flow_state::FlowState,
    ast_params: &[php_ast::owned::Param],
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) {
    for p in ast_params.iter() {
        let name_str = p.name.as_deref().unwrap_or("");
        let name = name_str.trim_start_matches('$');
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, p.span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, p.span.end, source_map);
        ctx.record_var_location(name, line, col_start, line_end, col_end);
        // Promoted properties (`public/protected/private $param`) are implicitly
        // assigned to `$this->prop` by PHP — mark them read so that constructors
        // with an empty body don't trigger UnusedParam.
        if p.visibility.is_some() {
            ctx.read_vars.insert(Name::from(name));
        }
    }
}

mod docblock_hint_compat {
    //! Coarse compatibility between a docblock type and a native hint.
    //!
    //! `is_subtype` is too strict for docblock refinement types
    //! (`literal-string`, `pure-callable`, `non-empty-list<...>` …), so the
    //! mismatch check uses PHP type FAMILIES: a docblock atom conflicts with
    //! the hint only when its family has no overlap with any hint family.
    //! Object-vs-object comparisons additionally use `is_subtype`, but only
    //! when every named class is known (unknown names are usually templates).
    use mir_types::{Atomic, Type};

    pub const NULL: u32 = 1;
    pub const BOOL: u32 = 1 << 1;
    pub const INT: u32 = 1 << 2;
    pub const FLOAT: u32 = 1 << 3;
    pub const STRING: u32 = 1 << 4;
    pub const ARRAY: u32 = 1 << 5;
    pub const OBJECT: u32 = 1 << 6;
    pub const CALLABLE: u32 = 1 << 7;
    pub const ALL: u32 = u32::MAX;

    /// Family bits for one atom. `0` means "matches anything" (unknown /
    /// placeholder kinds stay silent).
    pub fn fam(a: &Atomic) -> u32 {
        match a {
            Atomic::TNull => NULL,
            Atomic::TBool | Atomic::TTrue | Atomic::TFalse => BOOL,
            Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TPositiveInt
            | Atomic::TNegativeInt
            | Atomic::TNonNegativeInt => INT,
            Atomic::TFloat | Atomic::TLiteralFloat(..) => FLOAT,
            Atomic::TString
            | Atomic::TLiteralString(_)
            | Atomic::TNonEmptyString
            | Atomic::TNumericString
            | Atomic::TInterfaceString
            | Atomic::TEnumString
            | Atomic::TTraitString
            | Atomic::TClassString(_) => STRING,
            Atomic::TCallableString => STRING | CALLABLE,
            Atomic::TArray { .. }
            | Atomic::TList { .. }
            | Atomic::TNonEmptyArray { .. }
            | Atomic::TNonEmptyList { .. }
            | Atomic::TKeyedArray { .. } => ARRAY,
            Atomic::TObject
            | Atomic::TNamedObject { .. }
            | Atomic::TSelf { .. }
            | Atomic::TStaticObject { .. }
            | Atomic::TParent { .. }
            | Atomic::TIntersection { .. } => OBJECT,
            Atomic::TClosure { .. } => OBJECT | CALLABLE,
            Atomic::TCallable { .. } => CALLABLE | STRING | ARRAY | OBJECT,
            Atomic::TScalar => BOOL | INT | FLOAT | STRING,
            Atomic::TNumeric => INT | FLOAT | STRING,
            Atomic::TNever => 0, // never ⊆ everything
            _ => 0,              // mixed / templates / conditional / unknown
        }
    }

    /// What a HINT atom accepts — slightly wider than its own family
    /// (PHP coerces int → float; a `callable` hint takes strings, arrays
    /// and invokable objects).
    fn hint_mask(a: &Atomic) -> u32 {
        match a {
            Atomic::TMixed => ALL,
            Atomic::TFloat => FLOAT | INT,
            Atomic::TCallable { .. } | Atomic::TCallableString => {
                CALLABLE | STRING | ARRAY | OBJECT
            }
            // `object` hint takes any object incl. closures.
            Atomic::TObject => OBJECT | CALLABLE,
            other => {
                let f = fam(other);
                if f == 0 {
                    ALL
                } else {
                    f
                }
            }
        }
    }

    pub fn hint_accepts_mask(hint: &Type) -> u32 {
        hint.types.iter().fold(0, |acc, a| acc | hint_mask(a))
    }
}

/// Whether the docblock type CONTRADICTS the native hint (vs merely refining
/// it). See [`docblock_hint_compat`] for the family model.
fn docblock_conflicts_with_hint(
    db: &dyn crate::db::MirDatabase,
    doc_ty: &mir_types::Type,
    hint_ty: &mir_types::Type,
) -> bool {
    use docblock_hint_compat as fc;
    use mir_types::Atomic;

    let mask = fc::hint_accepts_mask(hint_ty);
    if doc_ty.types.iter().any(|a| {
        let fa = fc::fam(a);
        // Docblock refinement types the parser doesn't model become bare
        // named objects ("literal-string", "non-falsy-string", `C::class`
        // constant refs) — an OBJECT-family atom only counts when the class
        // actually exists, otherwise it is an unknown refinement: stay silent.
        if fa == fc::OBJECT {
            if let Atomic::TNamedObject { fqcn, .. } = a {
                if crate::db::find_class_like(db, crate::db::Fqcn::new(db, *fqcn)).is_none() {
                    return false;
                }
            }
        }
        fa != 0 && fa & mask == 0
    }) {
        return true;
    }

    // Object-vs-object precision: both sides purely class types with every
    // name known → inheritance-aware subtype check (catches `A&C` vs `A&B`).
    // Unknown names are usually unresolved templates — stay silent.
    fn collect_names(t: &mir_types::Type, out: &mut Vec<mir_types::Name>) -> bool {
        for a in &t.types {
            match a {
                Atomic::TNamedObject { fqcn, type_params } if type_params.is_empty() => {
                    out.push(*fqcn)
                }
                Atomic::TIntersection { parts } => {
                    for p in parts.iter() {
                        if !collect_names(p, out) {
                            return false;
                        }
                    }
                }
                _ => return false,
            }
        }
        true
    }
    let mut names = Vec::new();
    if !doc_ty.types.is_empty()
        && !hint_ty.types.is_empty()
        && collect_names(doc_ty, &mut names)
        && collect_names(hint_ty, &mut names)
        && names
            .iter()
            .all(|n| crate::db::find_class_like(db, crate::db::Fqcn::new(db, *n)).is_some())
    {
        return !crate::subtype::is_subtype(db, doc_ty, hint_ty);
    }
    false
}

/// Whether a docblock type still contains placeholders that cannot be
/// compared against a native hint here: template params, self/static/parent,
/// conditional types, or bare names matching a declared template.
fn docblock_type_unresolvable(ty: &mir_types::Type, template_names: &[&str]) -> bool {
    ty.types.iter().any(|a| match a {
        mir_types::Atomic::TTemplateParam { .. }
        | mir_types::Atomic::TSelf { .. }
        | mir_types::Atomic::TStaticObject { .. }
        | mir_types::Atomic::TParent { .. }
        | mir_types::Atomic::TConditional { .. } => true,
        mir_types::Atomic::TNamedObject { fqcn, type_params } => {
            (type_params.is_empty()
                && !fqcn.contains('\\')
                && (template_names.contains(&fqcn.as_str())
                    || fqcn.as_str().eq_ignore_ascii_case("static")
                    || fqcn.as_str().eq_ignore_ascii_case("self")))
                || !type_params.is_empty()
        }
        _ => false,
    })
}

/// Tight span for the function name in a `function name(...)` header. The
/// owned `Ident` carries no span, so search backward from the first param /
/// body for the last `name` occurrence; fall back to the body span.
fn fn_header_name_span(source: &str, decl: &php_ast::owned::FunctionDecl) -> php_ast::Span {
    let anchor = decl
        .params
        .first()
        .map(|p| p.span.start)
        .unwrap_or(decl.body.span.start) as usize;
    let anchor = anchor.min(source.len());
    let fallback = php_ast::Span {
        start: decl.body.span.start,
        end: decl.body.span.start + 1,
    };
    let Some(name) = decl.name.as_deref() else {
        return fallback;
    };
    if name.is_empty() {
        return fallback;
    }
    let search_start = anchor.saturating_sub(name.len() + 256);
    // Align to UTF-8 char boundaries to avoid a panic on multi-byte characters.
    let search_start = (search_start..=anchor.min(source.len()))
        .find(|&i| source.is_char_boundary(i))
        .unwrap_or(anchor);
    let anchor = (anchor..=source.len())
        .find(|&i| source.is_char_boundary(i))
        .unwrap_or(source.len());
    match source[search_start..anchor].rfind(name) {
        Some(rel) => {
            let start = (search_start + rel) as u32;
            php_ast::Span {
                start,
                end: start + name.len() as u32,
            }
        }
        None => fallback,
    }
}

/// Return the tight byte-offset span for only the `$name` token in a
/// parameter declaration. Falls back to the full param span when not found.
fn param_name_span(source: &str, p: &php_ast::owned::Param) -> php_ast::Span {
    let Some(raw) = p.name.as_deref() else {
        return p.span;
    };
    let bare = raw.trim_start_matches('$');
    let range_start = p.span.start as usize;
    let range_end = (p.span.end as usize).min(source.len());
    let slice = &source[range_start..range_end];
    let needle = format!("${bare}");
    if let Some(rel) = slice.find(needle.as_str()) {
        let start = p.span.start + rel as u32;
        php_ast::Span {
            start,
            end: start + needle.len() as u32,
        }
    } else {
        p.span
    }
}

/// Push one `Variable` symbol per parameter declaration into `all_symbols`.
/// Called immediately after [`seed_param_locations`] at every function/method body entry.
fn record_param_symbols(
    all_symbols: &mut Vec<ResolvedSymbol>,
    file: &Arc<str>,
    source: &str,
    ast_params: &[php_ast::owned::Param],
    ctx: &crate::flow_state::FlowState,
) {
    use crate::symbol::ReferenceKind;
    for p in ast_params {
        let Some(raw) = p.name.as_deref() else {
            continue;
        };
        let bare = raw.trim_start_matches('$');
        let span = param_name_span(source, p);
        let ty = ctx.get_var(bare);
        all_symbols.push(ResolvedSymbol {
            file: file.clone(),
            span,
            expr_span: None,
            kind: ReferenceKind::Variable(Arc::from(bare)),
            resolved_type: ty,
        });
    }
}

pub(crate) fn check_use_decl_casing(
    use_decl: &php_ast::owned::UseDecl,
    db: &dyn crate::db::MirDatabase,
    file: &std::sync::Arc<str>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
    all_issues: &mut Vec<Issue>,
) {
    use php_ast::ast::UseKind;
    for item in use_decl.uses.iter() {
        let effective_kind = item.kind.unwrap_or(use_decl.kind);
        let full_name = crate::parser::name_to_string_owned(&item.name)
            .trim_start_matches('\\')
            .to_string();
        if full_name.is_empty() {
            continue;
        }
        let (line, col_start) =
            crate::diagnostics::offset_to_line_col(source, item.span.start, source_map);
        let (line_end, col_end) =
            crate::diagnostics::offset_to_line_col(source, item.span.end, source_map);
        let loc = mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end: col_end.max(col_start + 1),
        };
        match effective_kind {
            UseKind::Normal => {
                let here = crate::db::Fqcn::from_str(db, &full_name);
                if let Some(class) = crate::db::find_class_like(db, here) {
                    if let Some((used, canonical_str)) =
                        crate::fqcn_case_mismatch(&full_name, class.fqcn().as_ref())
                    {
                        all_issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::WrongCaseClass {
                                used,
                                canonical: canonical_str,
                            },
                            loc,
                        ));
                    }
                }
            }
            UseKind::Function => {
                let here = crate::db::Fqcn::from_str(db, &full_name);
                if let Some(func) = crate::db::find_function(db, here) {
                    if let Some((used, canonical_str)) =
                        crate::fqcn_case_mismatch(&full_name, func.fqn.as_ref())
                    {
                        all_issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::WrongCaseFunction {
                                used,
                                canonical: canonical_str,
                            },
                            loc,
                        ));
                    }
                }
            }
            UseKind::Const => {}
        }
    }
}

/// Returns `true` if `stmts` contains any `yield` expression at any depth,
/// NOT descending into nested function/closure/arrow-function bodies (those
/// are separate generators with their own contexts).
pub(crate) fn body_has_yield(stmts: &[php_ast::owned::Stmt]) -> bool {
    use php_ast::owned::visitor::{walk_owned_expr, walk_owned_stmt, OwnedVisitor};
    use php_ast::owned::{ExprKind, StmtKind};
    use std::ops::ControlFlow;

    struct YieldFinder;

    impl OwnedVisitor for YieldFinder {
        fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> ControlFlow<()> {
            match &stmt.kind {
                // These are separate declaration scopes — don't descend.
                StmtKind::Function(_)
                | StmtKind::Class(_)
                | StmtKind::Interface(_)
                | StmtKind::Trait(_)
                | StmtKind::Enum(_) => ControlFlow::Continue(()),
                _ => walk_owned_stmt(self, stmt),
            }
        }

        fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> ControlFlow<()> {
            match &expr.kind {
                ExprKind::Yield(_) => ControlFlow::Break(()),
                // Closures and arrow functions are separate function scopes.
                ExprKind::Closure(_) | ExprKind::ArrowFunction(_) => ControlFlow::Continue(()),
                _ => walk_owned_expr(self, expr),
            }
        }
    }

    let mut finder = YieldFinder;
    for stmt in stmts {
        if finder.visit_stmt(stmt).is_break() {
            return true;
        }
    }
    false
}

pub fn merge_return_types(return_types: &[Type]) -> Type {
    if return_types.is_empty() {
        return Type::single(mir_types::Atomic::TVoid);
    }
    return_types.iter().fold(Type::empty(), |mut acc, t| {
        acc.merge_with(t);
        acc
    })
}
