use rustc_hash::FxHashMap;
use std::sync::Arc;

use mir_issues::Issue;
use mir_types::{Symbol, Union};
use parking_lot::Mutex;

use crate::db::{resolve_name, MirDatabase};
use crate::diagnostics::{
    check_expr_for_undefined_classes, check_name_class, check_type_hint_classes,
    collect_type_hint_class_refs, emit_unused_params, emit_unused_variables,
};
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

#[derive(Clone)]
pub(crate) struct InferredTypes {
    pub(crate) functions: Vec<(Arc<str>, Union)>,
    pub(crate) methods: Vec<(Arc<str>, Arc<str>, Union)>,
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
    Option<Union>,
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
            name: Symbol::new(p.name.as_deref().unwrap_or("")),
            ty: None,
            has_default: p.default.is_some(),
            is_variadic: p.variadic,
            is_byref: p.by_ref,
            is_optional: p.default.is_some() || p.variadic,
        })
        .collect()
}

pub(crate) struct BodyAnalyzer<'a> {
    db: &'a dyn MirDatabase,
    php_version: PhpVersion,
    inference_only: bool,
    inferred_types: Arc<Mutex<InferredTypes>>,
}

impl<'a> BodyAnalyzer<'a> {
    pub(crate) fn new(db: &'a dyn MirDatabase, php_version: PhpVersion) -> Self {
        Self {
            db,
            php_version,
            inference_only: false,
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
            inference_only: true,
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

    fn record_function_inference(&self, fqn: &Arc<str>, inferred: &Union) {
        if self.inference_only {
            let mut types = self.inferred_types.lock();
            types.functions.push((fqn.clone(), inferred.clone()));
        }
    }

    fn record_method_inference(&self, fqcn: &str, name: &str, inferred: &Union) {
        if self.inference_only {
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
        if !self.inference_only {
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
        use php_ast::owned::StmtKind;

        let mut all_issues = Vec::new();
        let mut all_symbols = Vec::new();

        for stmt in program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl(
                        decl,
                        &file,
                        source,
                        source_map,
                        &mut all_issues,
                        &mut all_symbols,
                    );
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl(
                        decl,
                        &file,
                        source,
                        source_map,
                        &mut all_issues,
                        &mut all_symbols,
                    );
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl(decl, &file, source, source_map, &mut all_issues);
                }
                StmtKind::Interface(decl) => {
                    self.analyze_interface_decl(decl, &file, source, source_map, &mut all_issues);
                }
                StmtKind::Trait(decl) => {
                    self.analyze_trait_decl(
                        decl,
                        &file,
                        source,
                        source_map,
                        &mut all_issues,
                        &mut all_symbols,
                    );
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::owned::NamespaceBody::Braced(stmts) = &ns.body {
                        for inner in stmts.iter() {
                            match &inner.kind {
                                StmtKind::Function(decl) => {
                                    self.analyze_fn_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                        &mut all_symbols,
                                    );
                                }
                                StmtKind::Class(decl) => {
                                    self.analyze_class_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                        &mut all_symbols,
                                    );
                                }
                                StmtKind::Enum(decl) => {
                                    self.analyze_enum_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                    );
                                }
                                StmtKind::Interface(decl) => {
                                    self.analyze_interface_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                    );
                                }
                                StmtKind::Trait(decl) => {
                                    self.analyze_trait_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                        &mut all_symbols,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Analyze top-level executable statements in global scope. The
        // inference-only sweep only primes function/method return types; top-
        // level diagnostics and references are produced by the main sweep.
        if !self.inference_only {
            use crate::context::Context;
            use crate::stmt::StatementsAnalyzer;
            use mir_issues::IssueBuffer;

            let mut ctx = Context::new();
            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                &mut all_symbols,
                self.php_version,
                self.inference_only,
            );
            for stmt in program.stmts.iter() {
                match &stmt.kind {
                    StmtKind::Function(_)
                    | StmtKind::Class(_)
                    | StmtKind::Enum(_)
                    | StmtKind::Interface(_)
                    | StmtKind::Trait(_)
                    | StmtKind::Namespace(_)
                    | StmtKind::Use(_)
                    | StmtKind::Declare(_) => {}
                    _ => {
                        sa.analyze_stmt(stmt, &mut ctx);
                    }
                }
            }
            drop(sa);
            all_issues.extend(buf.into_issues());
        }

        (all_issues, all_symbols)
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
        for stmt in program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Function(decl) => {
                    self.analyze_fn_decl_typed(
                        decl,
                        &file,
                        source,
                        source_map,
                        &mut all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Class(decl) => {
                    self.analyze_class_decl_typed(
                        decl,
                        &file,
                        source,
                        source_map,
                        &mut all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Enum(decl) => {
                    self.analyze_enum_decl(decl, &file, source, source_map, &mut all_issues);
                }
                StmtKind::Interface(decl) => {
                    self.analyze_interface_decl(decl, &file, source, source_map, &mut all_issues);
                }
                StmtKind::Trait(decl) => {
                    self.analyze_trait_decl_typed(
                        decl,
                        &file,
                        source,
                        source_map,
                        &mut all_issues,
                        type_envs,
                        all_symbols,
                    );
                }
                StmtKind::Namespace(ns) => {
                    if let php_ast::owned::NamespaceBody::Braced(stmts) = &ns.body {
                        for inner in stmts.iter() {
                            match &inner.kind {
                                StmtKind::Function(decl) => {
                                    self.analyze_fn_decl_typed(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                        type_envs,
                                        all_symbols,
                                    );
                                }
                                StmtKind::Class(decl) => {
                                    self.analyze_class_decl_typed(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                        type_envs,
                                        all_symbols,
                                    );
                                }
                                StmtKind::Enum(decl) => {
                                    self.analyze_enum_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                    );
                                }
                                StmtKind::Interface(decl) => {
                                    self.analyze_interface_decl(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                    );
                                }
                                StmtKind::Trait(decl) => {
                                    self.analyze_trait_decl_typed(
                                        decl,
                                        &file,
                                        source,
                                        source_map,
                                        &mut all_issues,
                                        type_envs,
                                        all_symbols,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Analyze top-level executable statements in global scope.
        {
            use crate::context::Context;
            use crate::stmt::StatementsAnalyzer;
            use mir_issues::IssueBuffer;

            let mut ctx = Context::new();
            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.inference_only,
            );
            for stmt in program.stmts.iter() {
                match &stmt.kind {
                    StmtKind::Function(_)
                    | StmtKind::Class(_)
                    | StmtKind::Enum(_)
                    | StmtKind::Interface(_)
                    | StmtKind::Trait(_)
                    | StmtKind::Namespace(_)
                    | StmtKind::Use(_)
                    | StmtKind::Declare(_) => {}
                    _ => {
                        sa.analyze_stmt(stmt, &mut ctx);
                    }
                }
            }
            drop(sa);
            all_issues.extend(buf.into_issues());
        }

        all_issues
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_fn_decl(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
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
        use crate::context::Context;
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

        let mut ctx = Context::for_method_with_templates(
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
            all_symbols,
            self.php_version,
            self.inference_only,
        );
        sa.analyze_stmts(&decl.body, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }

    /// Pure entry point: run the same analysis as [`Self::analyze_fn_decl`] for
    /// one function decl, but return the result instead of mutating
    /// caller-owned buffers. Used by the `infer_function` salsa tracked query.
    ///
    /// `ResolvedSymbol`s observed during the walk are intentionally dropped —
    /// symbols are re-walked on demand to keep the cache small.
    ///
    /// **Constraint:** this method drains `db.take_pending_ref_locs()` at entry
    /// to isolate the refs produced by this call. Don't invoke from a context
    /// where the same db handle has already-staged pending refs you care
    /// about — they will be discarded. The intended caller is the
    /// `infer_function` tracked query, which holds its own db handle per call.
    pub(crate) fn analyze_fn_decl_pure(
        &self,
        decl: &php_ast::owned::FunctionDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> crate::db::FunctionInferenceResult {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        // Clear any previously-staged refs on this db handle so we capture
        // only what this function's walk produces.
        let _ = self.db.take_pending_ref_locs();

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

        let mut ctx = Context::for_method_with_templates(
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
            self.inference_only,
        );
        sa.analyze_stmts(&decl.body, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, &mut issues);
        emit_unused_variables(&ctx, file, &mut issues);
        issues.extend(buf.into_issues());

        let ref_locs = self.db.take_pending_ref_locs();

        crate::db::FunctionInferenceResult {
            issues,
            ref_locs,
            return_type: Some(inferred),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_class_decl(
        &self,
        decl: &php_ast::owned::ClassDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

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

        for member in decl.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    self.check_and_record_type_hint_classes(
                        hint, file, source, source_map, all_issues,
                    );
                }
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
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

            if method.params.iter().any(|p| p.default.is_some()) {
                let mut buf = IssueBuffer::new();
                let mut sa = StatementsAnalyzer::new(
                    self.db,
                    file.clone(),
                    source,
                    source_map,
                    &mut buf,
                    all_symbols,
                    self.php_version,
                    self.inference_only,
                );
                let mut default_ctx = Context::new();
                default_ctx.self_fqcn = Some(Arc::from(fqcn));
                default_ctx.parent_fqcn = parent_fqcn.clone();
                default_ctx.static_fqcn = Some(Arc::from(fqcn));
                for p in method.params.iter() {
                    if let Some(default) = &p.default {
                        let mut ea = sa.expr_analyzer(&default_ctx);
                        let _ = ea.analyze(default, &mut default_ctx);
                    }
                }
                drop(sa);
                all_issues.extend(buf.into_issues());
            }

            let Some(body) = &method.body else { continue };
            let method_name = method.name.as_deref().unwrap_or("");

            let (params, return_ty, template_params, declared_throws) =
                method_chain_signature(self.db, fqcn, method_name);

            let is_ctor = method_name == "__construct";
            let mut ctx = Context::for_method_with_templates(
                &params,
                return_ty,
                declared_throws,
                Some(Arc::from(fqcn)),
                parent_fqcn.clone(),
                Some(Arc::from(fqcn)),
                false,
                is_ctor,
                method.is_static,
                Some(&template_params),
            );
            seed_param_locations(&mut ctx, &method.params, source, source_map);

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.inference_only,
            );
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            emit_unused_params(&params, &ctx, method_name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method_name, &inferred);
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
        use crate::context::Context;
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

        let mut ctx = Context::for_function(
            &params,
            return_ty,
            declared_throws,
            None,
            None,
            None,
            false,
            true,
        );
        seed_param_locations(&mut ctx, &decl.params, source, source_map);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.db,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
            self.inference_only,
        );
        sa.analyze_stmts(&decl.body, &mut ctx);
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
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

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

        for member in decl.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    self.check_and_record_type_hint_classes(
                        hint, file, source, source_map, all_issues,
                    );
                }
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
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

            if method.params.iter().any(|p| p.default.is_some()) {
                let mut buf = IssueBuffer::new();
                let mut sa = StatementsAnalyzer::new(
                    self.db,
                    file.clone(),
                    source,
                    source_map,
                    &mut buf,
                    all_symbols,
                    self.php_version,
                    self.inference_only,
                );
                let mut default_ctx = Context::new();
                default_ctx.self_fqcn = Some(Arc::from(fqcn));
                default_ctx.parent_fqcn = parent_fqcn.clone();
                default_ctx.static_fqcn = Some(Arc::from(fqcn));
                for p in method.params.iter() {
                    if let Some(default) = &p.default {
                        let mut ea = sa.expr_analyzer(&default_ctx);
                        let _ = ea.analyze(default, &mut default_ctx);
                    }
                }
                drop(sa);
                all_issues.extend(buf.into_issues());
            }

            let Some(body) = &method.body else { continue };
            let method_name = method.name.as_deref().unwrap_or("");

            let (params, return_ty, _, declared_throws) =
                method_chain_signature(self.db, fqcn, method_name);

            let is_ctor = method_name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
                declared_throws,
                Some(Arc::from(fqcn)),
                parent_fqcn.clone(),
                Some(Arc::from(fqcn)),
                false,
                is_ctor,
                method.is_static,
            );
            seed_param_locations(&mut ctx, &method.params, source, source_map);

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.inference_only,
            );
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            type_envs.insert(
                crate::type_env::ScopeId::Method {
                    class: Arc::from(fqcn),
                    method: Arc::from(method_name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );

            emit_unused_params(&params, &ctx, method_name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method_name, &inferred);
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
        let trait_locs: Vec<(Arc<str>, mir_codebase::storage::Location)> =
            class.trait_use_locations().to_vec();
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
    fn analyze_trait_decl(
        &self,
        decl: &php_ast::owned::TraitDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved = resolve_name(self.db, file.as_ref(), decl.name.as_deref().unwrap_or(""));
        let fqcn: &str = &resolved;

        for member in decl.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    self.check_and_record_type_hint_classes(
                        hint, file, source, source_map, all_issues,
                    );
                }
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
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

            let Some(body) = &method.body else { continue };
            let method_name = method.name.as_deref().unwrap_or("");

            let (params, return_ty, _, declared_throws) =
                method_chain_signature(self.db, fqcn, method_name);

            let is_ctor = method_name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
                declared_throws,
                Some(Arc::from(fqcn)),
                None,
                Some(Arc::from(fqcn)),
                false,
                is_ctor,
                method.is_static,
            );
            seed_param_locations(&mut ctx, &method.params, source, source_map);

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.inference_only,
            );
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            emit_unused_params(&params, &ctx, method_name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method_name, &inferred);
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
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved = resolve_name(self.db, file.as_ref(), decl.name.as_deref().unwrap_or(""));
        let fqcn: &str = &resolved;

        for member in decl.members.iter() {
            if let php_ast::owned::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    self.check_and_record_type_hint_classes(
                        hint, file, source, source_map, all_issues,
                    );
                }
                continue;
            }
            let php_ast::owned::ClassMemberKind::Method(method) = &member.kind else {
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

            let Some(body) = &method.body else { continue };
            let method_name = method.name.as_deref().unwrap_or("");

            let (params, return_ty, _, declared_throws) =
                method_chain_signature(self.db, fqcn, method_name);

            let is_ctor = method_name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
                declared_throws,
                Some(Arc::from(fqcn)),
                None,
                Some(Arc::from(fqcn)),
                false,
                is_ctor,
                method.is_static,
            );
            seed_param_locations(&mut ctx, &method.params, source, source_map);

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.db,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
                self.inference_only,
            );
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            type_envs.insert(
                crate::type_env::ScopeId::Method {
                    class: Arc::from(fqcn),
                    method: Arc::from(method_name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );

            emit_unused_params(&params, &ctx, method_name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method_name, &inferred);
        }
    }

    fn analyze_enum_decl(
        &self,
        decl: &php_ast::owned::EnumDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
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
        for member in decl.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
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
        }
    }

    fn analyze_interface_decl(
        &self,
        decl: &php_ast::owned::InterfaceDecl,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
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
        for member in decl.members.iter() {
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
        }
    }
}

/// Seed `ctx.var_locations` for function/method parameters using their AST spans.
fn seed_param_locations(
    ctx: &mut crate::context::Context,
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
    }
}

pub fn merge_return_types(return_types: &[Union]) -> Union {
    if return_types.is_empty() {
        return Union::single(mir_types::Atomic::TVoid);
    }
    return_types.iter().fold(Union::empty(), |mut acc, t| {
        acc.merge_with(t);
        acc
    })
}
