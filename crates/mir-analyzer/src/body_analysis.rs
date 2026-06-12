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
    if let Some((_, storage)) =
        crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, fqcn), method_name)
    {
        return (
            Arc::clone(&storage.params),
            storage.return_type.as_deref().cloned(),
            storage.template_params.clone(),
            Arc::from(storage.throws.as_slice()),
        );
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

    /// body analysis: walk all function/method bodies in one file, return issues, and
    /// write inferred return types back to the codebase.
    pub(crate) fn analyze_bodies(
        &self,
        program: &php_ast::owned::Program,
        file: Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> (Vec<Issue>, Vec<ResolvedSymbol>) {
        let mut all_issues = Vec::new();
        let mut all_symbols = Vec::new();

        if self.mode == AnalysisMode::Full {
            check_duplicate_declarations(
                &program.stmts,
                &file,
                source,
                source_map,
                &mut all_issues,
            );
        }

        self.analyze_top_level_stmts(
            &program.stmts,
            &file,
            source,
            source_map,
            &mut all_issues,
            &mut all_symbols,
        );

        // Analyze top-level executable statements in global scope. The
        // inference-only sweep only primes function/method return types; top-
        // level diagnostics and references are produced by the main sweep.
        self.analyze_global_exec(
            program,
            &file,
            source,
            source_map,
            &mut all_issues,
            &mut all_symbols,
        );

        (all_issues, all_symbols)
    }

    /// Analyze top-level executable statements in global scope (Full mode
    /// only). Extracted from [`Self::analyze_bodies`] so the per-scope
    /// tracked query can run it as its own scope.
    pub(crate) fn analyze_global_exec(
        &self,
        program: &php_ast::owned::Program,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::StmtKind;
        if self.mode != AnalysisMode::Full {
            return;
        }
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let mut ctx = FlowState::new();
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
            self.mode,
        );
        for stmt in program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(_)
                | StmtKind::Class(_)
                | StmtKind::Enum(_)
                | StmtKind::Interface(_)
                | StmtKind::Trait(_)
                | StmtKind::Namespace(_)
                | StmtKind::Use(_) => {}
                // Process Declare so that `declare(strict_types=1)` updates
                // ctx.strict_types before later executable stmts are analyzed.
                _ => {
                    sa.analyze_stmt(stmt, &mut ctx);
                }
            }
        }
        drop(sa);
        crate::diagnostics::emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());
    }

    /// Like `analyze_bodies` but also populates `type_envs` with per-scope type environments.
    pub(crate) fn analyze_bodies_typed(
        &self,
        program: &php_ast::owned::Program,
        file: Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) -> Vec<Issue> {
        use php_ast::owned::StmtKind;
        let mut all_issues = Vec::new();
        self.analyze_top_level_stmts_typed(
            &program.stmts,
            &file,
            source,
            source_map,
            &mut all_issues,
            type_envs,
            all_symbols,
        );

        // Analyze top-level executable statements in global scope.
        {
            use crate::flow_state::FlowState;
            use crate::stmt::StatementsAnalyzer;
            use mir_issues::IssueBuffer;

            let mut ctx = FlowState::new();
            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.mode,
            );
            for stmt in program.stmts.iter() {
                match &stmt.kind {
                    StmtKind::Function(_)
                    | StmtKind::Class(_)
                    | StmtKind::Enum(_)
                    | StmtKind::Interface(_)
                    | StmtKind::Trait(_)
                    | StmtKind::Namespace(_)
                    | StmtKind::Use(_) => {}
                    _ => {
                        sa.analyze_stmt(stmt, &mut ctx);
                    }
                }
            }
            drop(sa);
            crate::diagnostics::emit_unused_variables(&ctx, &file, &mut all_issues);
            all_issues.extend(buf.into_issues());
        }

        all_issues
    }

    fn analyze_top_level_stmts(
        &self,
        stmts: &[php_ast::owned::Stmt],
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::StmtKind;
        for stmt in stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl(decl, file, source, source_map, all_issues, all_symbols);
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        all_symbols,
                    );
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl(decl, file, source, source_map, all_issues, all_symbols);
                }
                StmtKind::Interface(decl) => {
                    self.analyze_interface_decl(decl, file, source, source_map, all_issues);
                }
                StmtKind::Trait(decl) => {
                    self.analyze_trait_decl(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        all_symbols,
                    );
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                        self.analyze_top_level_stmts(
                            &inner.stmts,
                            file,
                            source,
                            source_map,
                            all_issues,
                            all_symbols,
                        );
                    }
                }
                StmtKind::Use(use_decl) => {
                    check_use_decl_casing(use_decl, self.db, file, source, source_map, all_issues);
                }
                _ => {}
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_top_level_stmts_typed(
        &self,
        stmts: &[php_ast::owned::Stmt],
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::StmtKind;
        for stmt in stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Interface(decl) => {
                    self.analyze_interface_decl(decl, file, source, source_map, all_issues);
                }
                StmtKind::Trait(decl) => {
                    self.analyze_trait_decl_typed(
                        decl,
                        file,
                        source,
                        source_map,
                        all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::owned::NamespaceBody::Braced(inner) = &ns.body {
                        self.analyze_top_level_stmts_typed(
                            &inner.stmts,
                            file,
                            source,
                            source_map,
                            all_issues,
                            type_envs,
                            all_symbols,
                        );
                    }
                }
                StmtKind::Use(use_decl) => {
                    check_use_decl_casing(use_decl, self.db, file, source, source_map, all_issues);
                }
                _ => {}
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn analyze_fn_decl(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_function_attributes(
            decl, self.db, file, source, source_map, all_issues,
        );
        let fn_name = decl.name.as_deref().unwrap_or("").to_string();
        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
            }
            if let Some(default_expr) = &param.default {
                check_expr_for_undefined_classes(
                    default_expr,
                    self.db,
                    file,
                    source,
                    source_map,
                    all_issues,
                    self.php_version,
                );
            }
        }
        if let Some(hint) = &decl.return_type {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
        }
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved = lookup_function_node_for_decl(self.db, file.as_ref(), &fn_name);
        let fqn = resolved.as_ref().map(|(f, _)| f.clone());
        #[allow(clippy::type_complexity)]
        let (params, return_ty, template_params, declared_throws): (
            Arc<[mir_codebase::FnParam]>,
            _,
            Vec<_>,
            Arc<[Arc<str>]>,
        ) = match &resolved {
            Some((_, storage)) => {
                if storage.params.len() == decl.params.len()
                    && storage
                        .params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| ap.name.as_deref().unwrap_or("") == cp.name.as_ref())
                {
                    (
                        Arc::clone(&storage.params),
                        storage.return_type.as_deref().cloned(),
                        storage.template_params.clone(),
                        Arc::from(storage.throws.as_slice()),
                    )
                } else {
                    (
                        Arc::from(ast_derived_fn_params(&decl.params)),
                        None,
                        vec![],
                        Arc::from([]),
                    )
                }
            }
            None => (
                Arc::from(ast_derived_fn_params(&decl.params)),
                None,
                vec![],
                Arc::from([]),
            ),
        };

        if self.mode == AnalysisMode::Full {
            self.emit_missing_fn_types(
                decl,
                resolved.as_ref().map(|(_, s)| s),
                file,
                source,
                source_map,
                all_issues,
            );
        }

        let declared_return = return_ty.clone();
        let mut ctx = FlowState::for_method_with_templates(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            false,
            false,
            true,
            Some(&template_params),
        );
        ctx.is_in_pure_fn = resolved.as_ref().map(|(_, s)| s.is_pure).unwrap_or(false);
        seed_param_locations(&mut ctx, &decl.params, source, source_map);
        record_param_symbols(all_symbols, file, source, &decl.params, &ctx);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
            self.mode,
        );
        ctx.is_generator = body_has_yield(&decl.body.stmts);
        sa.analyze_stmts(&decl.body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let body_diverges = ctx.diverges;
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if self.mode == AnalysisMode::Full && !ctx.is_generator {
            crate::diagnostics::check_missing_return(
                declared_return.as_ref(),
                body_diverges,
                &decl.body.span,
                file,
                source,
                source_map,
                all_issues,
            );
        }

        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }

    /// Missing type declarations (Psalm parity): a top-level function with
    /// neither a native hint nor a docblock type. `stored` carries the
    /// docblock-resolved types, so absent there + absent in the AST = missing.
    fn emit_missing_fn_types(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        stored: Option<&Arc<mir_codebase::storage::FunctionDef>>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        issues: &mut Vec<Issue>,
    ) {
        let fn_name = decl.name.as_deref().unwrap_or("");
        let stored_params_match = stored.is_some_and(|s| s.params.len() == decl.params.len());
        if decl.return_type.is_none()
            && stored.is_none_or(|s| s.return_type.is_none())
            && !fn_name.is_empty()
        {
            let span = fn_header_name_span(source, decl);
            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, span.start, source_map);
            let (line_end, col_end) =
                crate::diagnostics::offset_to_line_col(source, span.end, source_map);
            issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::MissingReturnType {
                    fn_name: fn_name.to_string(),
                },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: col_end.max(col_start + 1),
                },
            ));
        }
        for (i, ast_param) in decl.params.iter().enumerate() {
            let stored_ty_present = stored_params_match
                && stored.is_some_and(|s| s.params.get(i).is_some_and(|p| p.ty.is_some()));
            if ast_param.type_hint.is_none() && !stored_ty_present {
                let param_name = ast_param
                    .name
                    .as_deref()
                    .unwrap_or("")
                    .trim_start_matches('$')
                    .to_string();
                let span = param_name_span(source, ast_param);
                let (line, col_start) =
                    crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                let (line_end, col_end) =
                    crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::MissingParamType {
                        fn_name: fn_name.to_string(),
                        param: param_name,
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: col_end.max(col_start + 1),
                    },
                ));
            }
        }

        // Docblock signature mismatches (Psalm parity): a docblock type that
        // contradicts the native hint. The stored type is the docblock-resolved
        // one (collector prefers docblock and marks `from_docblock`); the hint
        // is converted and namespace-resolved here for the comparison.
        let Some(stored) = stored else { return };
        let template_names: Vec<&str> = stored
            .template_params
            .iter()
            .map(|tp| tp.name.as_ref())
            .collect();
        if let (Some(hint), Some(doc_ty)) = (&decl.return_type, stored.return_type.as_deref()) {
            if doc_ty.from_docblock
                && !docblock_type_unresolvable(doc_ty, &template_names)
                && !fn_name.is_empty()
            {
                let hint_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    crate::parser::type_from_hint_owned(hint, None),
                    self.db,
                    file.as_ref(),
                );
                if !hint_ty.is_mixed()
                    && !doc_ty.is_mixed()
                    && docblock_conflicts_with_hint(self.db, doc_ty, &hint_ty)
                {
                    let span = fn_header_name_span(source, decl);
                    let (line, col_start) =
                        crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                    let (line_end, col_end) =
                        crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                    issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::MismatchingDocblockReturnType {
                            declared: doc_ty.to_string(),
                            inferred: hint_ty.to_string(),
                        },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    ));
                }
            }
        }
        // Param docblock types are not flagged `from_docblock` in storage, so
        // re-parse the doc comment to know which params have a docblock type.
        let doc = decl
            .doc_comment
            .as_ref()
            .map(|c| crate::parser::DocblockParser::parse(&c.text))
            .unwrap_or_default();
        {
            for ast_param in decl.params.iter() {
                let raw_name = ast_param.name.as_deref().unwrap_or_default();
                let (Some(hint), Some(doc_raw)) =
                    (&ast_param.type_hint, doc.get_param_type(raw_name))
                else {
                    continue;
                };
                let doc_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    doc_raw.clone(),
                    self.db,
                    file.as_ref(),
                );
                if docblock_type_unresolvable(&doc_ty, &template_names) {
                    continue;
                }
                let hint_ty = crate::expr::helpers::resolve_named_objects_in_union(
                    crate::parser::type_from_hint_owned(hint, None),
                    self.db,
                    file.as_ref(),
                );
                if hint_ty.is_mixed()
                    || doc_ty.is_mixed()
                    || !docblock_conflicts_with_hint(self.db, &doc_ty, &hint_ty)
                {
                    continue;
                }
                let param_name = ast_param
                    .name
                    .as_deref()
                    .unwrap_or("")
                    .trim_start_matches('$')
                    .to_string();
                let span = param_name_span(source, ast_param);
                let (line, col_start) =
                    crate::diagnostics::offset_to_line_col(source, span.start, source_map);
                let (line_end, col_end) =
                    crate::diagnostics::offset_to_line_col(source, span.end, source_map);
                issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::MismatchingDocblockParamType {
                        param: param_name,
                        declared: doc_ty.to_string(),
                        inferred: hint_ty.to_string(),
                    },
                    mir_issues::Location {
                        file: file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: col_end.max(col_start + 1),
                    },
                ));
            }
        }
    }

    /// Pure entry point: run the same analysis as [`Self::analyze_fn_decl`] for
    /// one function decl, but return the result instead of mutating
    /// caller-owned buffers. Used by the `infer_function` salsa tracked query.
    ///
    /// `ResolvedSymbol`s observed during the walk are intentionally dropped —
    /// symbols are re-walked on demand to keep the cache small.
    ///
    /// Ref-loc isolation: the walk is bracketed by a push/pop of a staging
    /// frame, so only refs produced by *this* call are returned — refs
    /// already staged on the handle (or recorded by a nested tracked query)
    /// are unaffected.
    pub(crate) fn analyze_fn_decl_pure(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> crate::db::FunctionInferenceResult {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        // Isolate this walk's refs in a fresh staging frame; popped at exit.
        self.db.push_ref_loc_frame();

        let mut issues: Vec<Issue> = Vec::new();
        let mut discarded_symbols: Vec<ResolvedSymbol> = Vec::new();

        let fn_name = decl.name.as_deref().unwrap_or("").to_string();
        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(
                    hint,
                    file,
                    source,
                    source_map,
                    &mut issues,
                );
            }
            if let Some(default_expr) = &param.default {
                check_expr_for_undefined_classes(
                    default_expr,
                    self.db,
                    file,
                    source,
                    source_map,
                    &mut issues,
                    self.php_version,
                );
            }
        }
        if let Some(hint) = &decl.return_type {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, &mut issues);
        }

        let resolved = lookup_function_node_for_decl(self.db, file.as_ref(), &fn_name);
        if self.mode == AnalysisMode::Full {
            self.emit_missing_fn_types(
                decl,
                resolved.as_ref().map(|(_, s)| s),
                file,
                source,
                source_map,
                &mut issues,
            );
        }
        #[allow(clippy::type_complexity)]
        let (params, return_ty, template_params, declared_throws): (
            Arc<[mir_codebase::FnParam]>,
            _,
            Vec<_>,
            Arc<[Arc<str>]>,
        ) = match &resolved {
            Some((_, storage))
                if storage.params.len() == decl.params.len()
                    && storage
                        .params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| ap.name.as_deref().unwrap_or("") == cp.name.as_ref()) =>
            {
                (
                    Arc::clone(&storage.params),
                    storage.return_type.as_deref().cloned(),
                    storage.template_params.clone(),
                    Arc::from(storage.throws.as_slice()),
                )
            }
            _ => (
                Arc::from(ast_derived_fn_params(&decl.params)),
                None,
                vec![],
                Arc::from([]),
            ),
        };

        let mut ctx = FlowState::for_method_with_templates(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            false,
            false,
            true,
            Some(&template_params),
        );
        seed_param_locations(&mut ctx, &decl.params, source, source_map);

        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            &mut discarded_symbols,
            self.php_version,
            self.mode,
        );
        ctx.is_generator = body_has_yield(&decl.body.stmts);
        sa.analyze_stmts(&decl.body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, &mut issues);
        emit_unused_variables(&ctx, file, &mut issues);
        issues.extend(buf.into_issues());

        let ref_locs = self.db.pop_ref_loc_frame();

        crate::db::FunctionInferenceResult {
            issues,
            ref_locs,
            return_type: Some(inferred),
        }
    }

    #[allow(clippy::too_many_arguments)]
    /// Property-member checks shared by the class and trait paths: type-hint
    /// class resolution when a hint is present, `MissingPropertyType`
    /// otherwise (Full mode).
    #[allow(clippy::too_many_arguments)]
    fn check_property_member(
        &self,
        prop: &php_ast::owned::PropertyDecl,
        member_span: &php_ast::Span,
        fqcn: &str,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        if let Some(hint) = &prop.type_hint {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
        } else if self.mode == AnalysisMode::Full {
            let prop_name = prop.name.as_deref().unwrap_or("").to_string();
            let (line, col_start) =
                crate::diagnostics::offset_to_line_col(source, member_span.start, source_map);
            let (line_end, col_end) =
                crate::diagnostics::offset_to_line_col(source, member_span.end, source_map);
            all_issues.push(mir_issues::Issue::new(
                mir_issues::IssueKind::MissingPropertyType {
                    class: fqcn.to_string(),
                    property: prop_name,
                },
                mir_issues::Location {
                    file: file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: col_end.max(col_start + 1),
                },
            ));
        }
    }

    /// Analyze one class-like member method: hint checks, optional parameter
    /// default-value analysis, FlowState construction, body statement
    /// analysis, unused-param/-var emission, optional return checks, and
    /// inference recording.
    ///
    /// One shared core replaces the six previously copy-pasted blocks
    /// (class / trait / enum × plain / typed). [`MethodScopeCx`] captures the
    /// container-kind divergences so each call site's behavior — including
    /// issue emission *order* — is reproduced exactly.
    #[allow(clippy::too_many_arguments)]
    fn analyze_method_scope(
        &self,
        method: &php_ast::owned::MethodDecl,
        cx: &MethodScopeCx,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
        type_envs: Option<&mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>>,
    ) {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let fqcn: &str = cx.fqcn.as_ref();

        for param in method.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &method.return_type {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
        }

        if cx.analyze_param_defaults && method.params.iter().any(|p| p.default.is_some()) {
            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.mode,
            );
            let mut default_ctx = FlowState::new();
            default_ctx.self_fqcn = Some(cx.fqcn.clone());
            default_ctx.parent_fqcn = cx.parent_fqcn.clone();
            default_ctx.static_fqcn = Some(cx.fqcn.clone());
            for p in method.params.iter() {
                if let Some(default) = &p.default {
                    let mut ea = sa.expr_analyzer(&default_ctx);
                    let _ = ea.analyze(default, &mut default_ctx);
                }
            }
            drop(sa);
            all_issues.extend(buf.into_issues());
        }

        let Some(body) = &method.body else { return };
        let method_name = method.name.as_deref().unwrap_or("");

        let (params, return_ty, template_params, declared_throws) =
            method_chain_signature(self.db, fqcn, method_name);

        let declared_return = return_ty.clone();
        let is_ctor = cx.detect_ctor && method_name == "__construct";
        let templates: Option<&[mir_codebase::storage::TemplateParam]> = if cx.with_templates {
            Some(&template_params)
        } else {
            None
        };
        let mut ctx = FlowState::for_method_with_templates(
            &params,
            return_ty,
            declared_throws,
            Some(cx.fqcn.clone()),
            cx.parent_fqcn.clone(),
            Some(cx.fqcn.clone()),
            false,
            is_ctor,
            method.is_static,
            templates,
        );
        // Set is_in_pure_fn if the method is annotated @pure.
        if let Some((_, method_storage)) = crate::db::find_method_in_chain(
            self.db,
            crate::db::Fqcn::from_str(self.db, fqcn),
            &method_name.to_ascii_lowercase(),
        ) {
            ctx.is_in_pure_fn = method_storage.is_pure;
        }

        seed_param_locations(&mut ctx, &method.params, source, source_map);
        record_param_symbols(all_symbols, file, source, &method.params, &ctx);

        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
            self.mode,
        );
        ctx.is_generator = body_has_yield(&body.stmts);
        sa.analyze_stmts(&body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        let body_diverges = ctx.diverges;
        drop(sa);

        if let Some(type_envs) = type_envs {
            type_envs.insert(
                crate::type_env::ScopeId::Method {
                    class: cx.fqcn.clone(),
                    method: Arc::from(method_name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );
        }

        emit_unused_params(&params, &ctx, method_name, file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if cx.check_returns && self.mode == AnalysisMode::Full && !is_ctor && !ctx.is_generator {
            crate::diagnostics::check_missing_return(
                declared_return.as_ref(),
                body_diverges,
                &body.span,
                file,
                source,
                source_map,
                all_issues,
            );
        }

        if cx.check_returns
            && self.mode == AnalysisMode::Full
            && method_name.eq_ignore_ascii_case("__tostring")
        {
            crate::diagnostics::check_to_string_return(
                fqcn,
                declared_return.as_ref(),
                &inferred,
                &body.span,
                file,
                source,
                source_map,
                all_issues,
            );
        }

        self.record_method_inference(fqcn, method_name, &inferred);
    }

    pub(crate) fn analyze_class_decl(
        &self,
        decl: &php_ast::owned::ClassDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_class_attributes(
            decl, self.db, file, source, source_map, all_issues,
        );

        let class_name_owned = decl
            .name
            .as_ref()
            .and_then(|i| i.as_deref())
            .unwrap_or("<anonymous>")
            .to_string();
        let class_name = class_name_owned.as_str();
        let resolved = resolve_name(self.db, file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let parent_fqcn =
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

        crate::attributes::check_parent_in_class_attrs(
            &decl.attributes,
            parent_fqcn.is_some(),
            file,
            source,
            source_map,
            all_issues,
        );

        if let Some(parent) = &decl.extends {
            crate::diagnostics::check_name_class_for_extends(
                parent,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }
        for iface in decl.implements.iter() {
            check_name_class(
                iface,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: parent_fqcn.clone(),
            detect_ctor: true,
            with_templates: true,
            check_returns: true,
            analyze_param_defaults: true,
        };
        for member in decl.body.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                self.check_property_member(
                    prop,
                    &member.span,
                    fqcn,
                    file,
                    source,
                    source_map,
                    all_issues,
                );
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                None,
            );
        }

        self.check_trait_constraints(fqcn, file, all_issues);
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_fn_decl_typed(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::flow_state::FlowState;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let fn_name = decl.name.as_deref().unwrap_or("").to_string();

        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
        }

        let resolved = lookup_function_node_for_decl(self.db, file.as_ref(), &fn_name);
        if self.mode == AnalysisMode::Full {
            self.emit_missing_fn_types(
                decl,
                resolved.as_ref().map(|(_, s)| s),
                file,
                source,
                source_map,
                all_issues,
            );
        }
        let fqn = resolved.as_ref().map(|(f, _)| f.clone());
        let (params, return_ty, declared_throws): (
            Arc<[mir_codebase::FnParam]>,
            _,
            Arc<[Arc<str>]>,
        ) = match &resolved {
            Some((_, storage)) => {
                if storage.params.len() == decl.params.len()
                    && storage
                        .params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| ap.name.as_deref().unwrap_or("") == cp.name.as_ref())
                {
                    (
                        Arc::clone(&storage.params),
                        storage.return_type.as_deref().cloned(),
                        Arc::from(storage.throws.as_slice()),
                    )
                } else {
                    (
                        Arc::from(ast_derived_fn_params(&decl.params)),
                        None,
                        Arc::from([]),
                    )
                }
            }
            None => (
                Arc::from(ast_derived_fn_params(&decl.params)),
                None,
                Arc::from([]),
            ),
        };

        let mut ctx = FlowState::for_function(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            false,
            true,
        );
        ctx.is_in_pure_fn = resolved.as_ref().map(|(_, s)| s.is_pure).unwrap_or(false);
        seed_param_locations(&mut ctx, &decl.params, source, source_map);
        record_param_symbols(all_symbols, file, source, &decl.params, &ctx);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
            self.mode,
        );
        ctx.is_generator = body_has_yield(&decl.body.stmts);
        sa.analyze_stmts(&decl.body.stmts, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        let scope_name = fqn.clone().unwrap_or_else(|| Arc::from(fn_name));
        type_envs.insert(
            crate::type_env::ScopeId::Function {
                file: file.clone(),
                name: scope_name,
            },
            crate::type_env::TypeEnv::new(ctx.vars.clone()),
        );

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_class_decl_typed(
        &self,
        decl: &php_ast::owned::ClassDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        let class_name_owned = decl
            .name
            .as_ref()
            .and_then(|i| i.as_deref())
            .unwrap_or("<anonymous>")
            .to_string();
        let class_name = class_name_owned.as_str();
        let resolved = resolve_name(self.db, file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let parent_fqcn =
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

        if let Some(parent) = &decl.extends {
            crate::diagnostics::check_name_class_for_extends(
                parent,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }
        for iface in decl.implements.iter() {
            check_name_class(
                iface,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: parent_fqcn.clone(),
            detect_ctor: true,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: true,
        };
        for member in decl.body.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                self.check_property_member(
                    prop,
                    &member.span,
                    fqcn,
                    file,
                    source,
                    source_map,
                    all_issues,
                );
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                Some(&mut *type_envs),
            );
        }

        self.check_trait_constraints(fqcn, file, all_issues);
    }

    /// Emit `InvalidTraitUse` issues if this class violates any `@psalm-require-extends` /
    /// `@psalm-require-implements` constraint declared on the traits it uses.
    fn check_trait_constraints(&self, fqcn: &str, file: &Arc<str>, all_issues: &mut Vec<Issue>) {
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let Some(class) = crate::db::find_class_like(self.db, here) else {
            return;
        };
        let trait_list: Vec<Arc<str>> = class.class_traits().to_vec();
        let trait_locs: Vec<(Arc<str>, mir_types::Location)> = class.trait_use_locations().to_vec();
        let class_all_parents: Vec<Arc<str>> = crate::db::class_ancestors(self.db, here).0;

        for trait_fqcn in trait_list.iter() {
            let tr_short: Arc<str> = trait_fqcn
                .rsplit('\\')
                .next()
                .map(Arc::from)
                .unwrap_or_else(|| trait_fqcn.clone());

            let make_loc = || {
                trait_locs
                    .iter()
                    .find(|(f, _)| f.as_ref() == trait_fqcn.as_ref())
                    .map(|(_, loc)| mir_issues::Location {
                        file: loc.file.clone(),
                        line: loc.line,
                        line_end: loc.line_end,
                        col_start: loc.col_start,
                        col_end: loc.col_end,
                    })
                    .unwrap_or_else(|| mir_issues::Location {
                        file: file.clone(),
                        line: 1,
                        line_end: 1,
                        col_start: 0,
                        col_end: 0,
                    })
            };

            let trait_here = crate::db::Fqcn::from_str(self.db, trait_fqcn.as_ref());
            let trait_class = match crate::db::find_class_like(self.db, trait_here) {
                None => {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::UndefinedTrait {
                            name: tr_short.to_string(),
                        },
                        make_loc(),
                    ));
                    continue;
                }
                Some(c) => c,
            };

            if !trait_class.is_trait() {
                let (article, kind) = if trait_class.is_interface() {
                    ("an", "interface")
                } else if trait_class.is_enum() {
                    ("an", "enum")
                } else {
                    ("a", "class")
                };
                all_issues.push(mir_issues::Issue::new(
                    mir_issues::IssueKind::InvalidTraitUse {
                        trait_name: tr_short.to_string(),
                        reason: format!("{tr_short} is {article} {kind}, not a trait"),
                    },
                    make_loc(),
                ));
                continue;
            }

            let (req_ext, req_impl): (Vec<Arc<str>>, Vec<Arc<str>>) = match &trait_class {
                crate::db::ClassLike::Trait(t) => {
                    (t.require_extends.to_vec(), t.require_implements.to_vec())
                }
                _ => (vec![], vec![]),
            };
            if req_ext.is_empty() && req_impl.is_empty() {
                continue;
            }

            for req in req_ext.iter() {
                let satisfies = fqcn == req.as_ref()
                    || class_all_parents.iter().any(|p| p.as_ref() == req.as_ref());
                if !satisfies {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::InvalidTraitUse {
                            trait_name: tr_short.to_string(),
                            reason: format!(
                                "Class {fqcn} uses trait {tr_short} but does not extend {req}"
                            ),
                        },
                        make_loc(),
                    ));
                }
            }

            for req in req_impl.iter() {
                let satisfies = class_all_parents.iter().any(|p| p.as_ref() == req.as_ref());
                if !satisfies {
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::InvalidTraitUse {
                            trait_name: tr_short.to_string(),
                            reason: format!(
                                "Class {fqcn} uses trait {tr_short} but does not implement {req}"
                            ),
                        },
                        make_loc(),
                    ));
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn analyze_trait_decl(
        &self,
        decl: &php_ast::owned::TraitDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        crate::attributes::check_trait_attributes(
            decl, self.db, file, source, source_map, all_issues,
        );

        let resolved = resolve_name(self.db, file.as_ref(), decl.name.as_deref().unwrap_or(""));
        let fqcn: &str = &resolved;

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: true,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: false,
        };
        for member in decl.body.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                self.check_property_member(
                    prop,
                    &member.span,
                    fqcn,
                    file,
                    source,
                    source_map,
                    all_issues,
                );
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                None,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_trait_decl_typed(
        &self,
        decl: &php_ast::owned::TraitDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        let resolved = resolve_name(self.db, file.as_ref(), decl.name.as_deref().unwrap_or(""));
        let fqcn: &str = &resolved;

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: true,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: false,
        };
        for member in decl.body.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                self.check_property_member(
                    prop,
                    &member.span,
                    fqcn,
                    file,
                    source,
                    source_map,
                    all_issues,
                );
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                Some(&mut *type_envs),
            );
        }
    }

    pub(crate) fn analyze_enum_decl(
        &self,
        decl: &php_ast::owned::EnumDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::EnumMemberKind;

        for iface in decl.implements.iter() {
            check_name_class(
                iface,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }

        let enum_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let resolved = resolve_name(self.db, file.as_ref(), enum_name);
        let fqcn: &str = &resolved;

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: false,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: false,
        };
        for member in decl.body.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                None,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_enum_decl_typed(
        &self,
        decl: &php_ast::owned::EnumDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut rustc_hash::FxHashMap<crate::type_env::ScopeId, crate::type_env::TypeEnv>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use php_ast::owned::EnumMemberKind;

        // Single pass: same analysis as the untyped path, additionally
        // recording type environments for LSP hover/go-to-def. (Previously
        // this ran the full body analysis twice and discarded the second
        // run's issues — the shared core makes one run produce both.)
        for iface in decl.implements.iter() {
            check_name_class(
                iface,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }

        let enum_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let resolved = resolve_name(self.db, file.as_ref(), enum_name);
        let fqcn: &str = &resolved;

        let scope_cx = MethodScopeCx {
            fqcn: Arc::from(fqcn),
            parent_fqcn: None,
            detect_ctor: false,
            with_templates: false,
            check_returns: false,
            analyze_param_defaults: false,
        };
        for member in decl.body.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            self.analyze_method_scope(
                method,
                &scope_cx,
                file,
                source,
                source_map,
                all_issues,
                all_symbols,
                Some(&mut *type_envs),
            );
        }
    }

    pub(crate) fn analyze_interface_decl(
        &self,
        decl: &php_ast::owned::InterfaceDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        crate::attributes::check_interface_attributes(
            decl, self.db, file, source, source_map, all_issues,
        );
        use php_ast::owned::ClassMemberKind;
        for parent in decl.extends.iter() {
            check_name_class(
                parent,
                self.db,
                file,
                source,
                source_map,
                all_issues,
                self.php_version,
            );
        }
        let iface_name = decl.name.as_deref().unwrap_or("<anonymous>");
        let iface_fqcn = resolve_name(self.db, file.as_ref(), iface_name);
        let iface_fqcn_ref = crate::db::Fqcn::from_str(self.db, &iface_fqcn);

        for member in decl.body.members.iter() {
            let ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    self.check_and_record_type_hint_classes(
                        hint, file, source, source_map, all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                self.check_and_record_type_hint_classes(hint, file, source, source_map, all_issues);
            }

            let method_name = method.name.as_deref().unwrap_or("");
            let stored = crate::db::find_method_in_class(self.db, iface_fqcn_ref, method_name);

            if self.mode == AnalysisMode::Full {
                let stored_return = stored.as_ref().and_then(|m| m.return_type.as_deref());
                if method.return_type.is_none() && stored_return.is_none() {
                    let fn_name = format!("{iface_fqcn}::{method_name}");
                    let (line, col_start) = crate::diagnostics::offset_to_line_col(
                        source,
                        member.span.start,
                        source_map,
                    );
                    let (line_end, col_end) =
                        crate::diagnostics::offset_to_line_col(source, member.span.end, source_map);
                    all_issues.push(mir_issues::Issue::new(
                        mir_issues::IssueKind::MissingReturnType { fn_name },
                        mir_issues::Location {
                            file: file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    ));
                }

                if let Some(stored_method) = &stored {
                    let fn_name = format!("{iface_fqcn}::{method_name}");
                    for (ast_param, stored_param) in
                        method.params.iter().zip(stored_method.params.iter())
                    {
                        if ast_param.type_hint.is_none() && stored_param.ty.is_none() {
                            let param_name = ast_param
                                .name
                                .as_deref()
                                .unwrap_or("")
                                .trim_start_matches('$')
                                .to_string();
                            let span = param_name_span(source, ast_param);
                            let (line, col_start) = crate::diagnostics::offset_to_line_col(
                                source, span.start, source_map,
                            );
                            let (line_end, col_end) = crate::diagnostics::offset_to_line_col(
                                source, span.end, source_map,
                            );
                            all_issues.push(mir_issues::Issue::new(
                                mir_issues::IssueKind::MissingParamType {
                                    fn_name: fn_name.clone(),
                                    param: param_name,
                                },
                                mir_issues::Location {
                                    file: file.clone(),
                                    line,
                                    line_end,
                                    col_start,
                                    col_end: col_end.max(col_start + 1),
                                },
                            ));
                        }
                    }
                }
            }
        }
    }
}

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
                    let written_short = full_name.rsplit('\\').next().unwrap_or(full_name.as_str());
                    let canonical_short = class
                        .fqcn()
                        .rsplit('\\')
                        .next()
                        .unwrap_or(class.fqcn().as_ref());
                    if written_short != canonical_short
                        && written_short.eq_ignore_ascii_case(canonical_short)
                    {
                        all_issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::WrongCaseClass {
                                used: written_short.to_string(),
                                canonical: canonical_short.to_string(),
                            },
                            loc,
                        ));
                    }
                }
            }
            UseKind::Function => {
                let here = crate::db::Fqcn::from_str(db, &full_name);
                if let Some(func) = crate::db::find_function(db, here) {
                    let written_short = full_name.rsplit('\\').next().unwrap_or(full_name.as_str());
                    let canonical_short = func.fqn.rsplit('\\').next().unwrap_or(func.fqn.as_ref());
                    if written_short != canonical_short
                        && written_short.eq_ignore_ascii_case(canonical_short)
                    {
                        all_issues.push(mir_issues::Issue::new(
                            mir_issues::IssueKind::WrongCaseFunction {
                                used: written_short.to_string(),
                                canonical: canonical_short.to_string(),
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
