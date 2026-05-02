use std::sync::Arc;

use mir_issues::Issue;
use mir_types::Union;

use crate::db::{resolve_name_via_db, FunctionNode, InferredReturnTypes, MirDatabase};
use crate::diagnostics::{
    check_name_class, check_type_hint_classes, emit_unused_params, emit_unused_variables,
};
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

/// Resolve a function declaration's `FunctionNode` via the salsa db,
/// matching the pre-S5 fallback chain (qualified FQN → raw name →
/// short-name scan).  `None` if no active node matches.
fn lookup_function_node_for_decl(
    db: &dyn MirDatabase,
    file: &str,
    fn_name: &str,
) -> Option<FunctionNode> {
    let qualified = resolve_name_via_db(db, file, fn_name);
    if let Some(n) = db
        .lookup_function_node(qualified.as_str())
        .filter(|n| n.active(db))
    {
        return Some(n);
    }
    if let Some(n) = db.lookup_function_node(fn_name).filter(|n| n.active(db)) {
        return Some(n);
    }
    for fqn in db.active_function_node_fqns() {
        let short = fqn.as_ref().rsplit('\\').next().unwrap_or(fqn.as_ref());
        if short == fn_name {
            if let Some(n) = db
                .lookup_function_node(fqn.as_ref())
                .filter(|n| n.active(db))
            {
                return Some(n);
            }
        }
    }
    None
}

/// Build `FnParam`s directly from the declaration AST when no storage match is
/// available.  Defaults are typed as `mixed` since their value type isn't tracked.
fn ast_derived_fn_params<'arena, 'src>(
    params: &[php_ast::ast::Param<'arena, 'src>],
) -> Vec<mir_codebase::FnParam> {
    params
        .iter()
        .map(|p| mir_codebase::FnParam {
            name: Arc::from(p.name),
            ty: None,
            default: p.default.as_ref().map(|_| Union::mixed()),
            is_variadic: p.variadic,
            is_byref: p.by_ref,
            is_optional: p.default.is_some() || p.variadic,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pass2Driver
// ---------------------------------------------------------------------------

pub(crate) struct Pass2Driver<'a> {
    db: &'a dyn MirDatabase,
    php_version: PhpVersion,
    inference_only: bool,
    /// Optional buffer for inferred return types; populated by `analyze_*`
    /// during the priming sweep.  See [`InferredReturnTypes`] and
    /// `MirDb::commit_inferred_return_types`.  `None` means "skip the
    /// salsa-side commit" — the main sweep doesn't need to publish
    /// inferred types because the priming sweep already did.
    inferred_buffer: Option<&'a InferredReturnTypes>,
}

impl<'a> Pass2Driver<'a> {
    pub(crate) fn new(db: &'a dyn MirDatabase, php_version: PhpVersion) -> Self {
        Self {
            db,
            php_version,
            inference_only: false,
            inferred_buffer: None,
        }
    }

    pub(crate) fn new_inference_only(db: &'a dyn MirDatabase, php_version: PhpVersion) -> Self {
        Self {
            db,
            php_version,
            inference_only: true,
            inferred_buffer: None,
        }
    }

    /// Attach an inferred-return-type buffer.  Used during the priming
    /// sweep so workers record their inferred types for the post-sweep
    /// commit phase.
    pub(crate) fn with_inferred_buffer(mut self, buf: &'a InferredReturnTypes) -> Self {
        self.inferred_buffer = Some(buf);
        self
    }

    /// Push a function inference into the buffer (if attached).
    fn record_function_inference(&self, fqn: &Arc<str>, inferred: &Union) {
        if let Some(buf) = self.inferred_buffer {
            buf.push_function(fqn.clone(), inferred.clone());
        }
    }

    /// Push a method inference into the buffer (if attached).
    fn record_method_inference(&self, fqcn: &str, name: &str, inferred: &Union) {
        if let Some(buf) = self.inferred_buffer {
            buf.push_method(Arc::from(fqcn), Arc::from(name), inferred.clone());
        }
    }

    /// Pass 2: walk all function/method bodies in one file, return issues, and
    /// write inferred return types back to the codebase.
    pub(crate) fn analyze_bodies<'arena, 'src>(
        &self,
        program: &php_ast::ast::Program<'arena, 'src>,
        file: Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
    ) -> (Vec<Issue>, Vec<ResolvedSymbol>) {
        use php_ast::ast::StmtKind;

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
                    if let php_ast::ast::NamespaceBody::Braced(stmts) = &ns.body {
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
                    _ => sa.analyze_stmt(stmt, &mut ctx),
                }
            }
            drop(sa);
            all_issues.extend(buf.into_issues());
        }

        (all_issues, all_symbols)
    }

    /// Like `analyze_bodies` but also populates `type_envs` with per-scope type environments.
    pub(crate) fn analyze_bodies_typed<'arena, 'src>(
        &self,
        program: &php_ast::ast::Program<'arena, 'src>,
        file: Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) -> Vec<Issue> {
        use php_ast::ast::StmtKind;
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
                    if let php_ast::ast::NamespaceBody::Braced(stmts) = &ns.body {
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
                    _ => sa.analyze_stmt(stmt, &mut ctx),
                }
            }
            drop(sa);
            all_issues.extend(buf.into_issues());
        }

        all_issues
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_fn_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        let fn_name = decl.name;
        let body = &decl.body;
        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
        }
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let node_opt = lookup_function_node_for_decl(self.db, file.as_ref(), fn_name);
        let fqn = node_opt.map(|n| n.fqn(self.db));
        let (params, return_ty): (Vec<mir_codebase::FnParam>, _) = match node_opt {
            Some(n) => {
                let stored = n.params(self.db);
                if stored.len() == decl.params.len()
                    && stored
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| cp.name.as_ref() == ap.name)
                {
                    (stored.to_vec(), n.return_type(self.db))
                } else {
                    (ast_derived_fn_params(&decl.params), None)
                }
            }
            None => (ast_derived_fn_params(&decl.params), None),
        };

        let mut ctx = Context::for_function(&params, return_ty, None, None, None, false, true);
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
        sa.analyze_stmts(body, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        // Inferred return type → Salsa `FunctionNode::inferred_return_type`
        // via the parallel-safe buffer (committed serially after the
        // priming sweep returns).  See `InferredReturnTypes`.
        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_class_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::ClassDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let class_name = decl.name.unwrap_or("<anonymous>");
        let resolved = resolve_name_via_db(self.db, file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let parent_fqcn = self
            .db
            .lookup_class_node(fqcn)
            .and_then(|node| node.parent(self.db));

        if let Some(parent) = &decl.extends {
            check_name_class(parent, self.db, file, source, source_map, all_issues);
        }
        for iface in decl.implements.iter() {
            check_name_class(iface, self.db, file, source, source_map, all_issues);
        }

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = crate::db::lookup_method_in_chain(self.db, fqcn, method.name)
                .map(|n| (n.params(self.db).to_vec(), n.return_type(self.db)))
                .unwrap_or_default();

            let is_ctor = method.name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
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

            emit_unused_params(&params, &ctx, method.name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method.name, &inferred);
        }

        self.check_trait_constraints(fqcn, file, all_issues);
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_fn_decl_typed<'arena, 'src>(
        &self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let fn_name = decl.name;
        let body = &decl.body;

        for param in decl.params.iter() {
            if let Some(hint) = &param.type_hint {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
        }

        let node_opt = lookup_function_node_for_decl(self.db, file.as_ref(), fn_name);
        let fqn = node_opt.map(|n| n.fqn(self.db));
        let (params, return_ty): (Vec<mir_codebase::FnParam>, _) = match node_opt {
            Some(n) => {
                let stored = n.params(self.db);
                if stored.len() == decl.params.len()
                    && stored
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| cp.name.as_ref() == ap.name)
                {
                    (stored.to_vec(), n.return_type(self.db))
                } else {
                    (ast_derived_fn_params(&decl.params), None)
                }
            }
            None => (ast_derived_fn_params(&decl.params), None),
        };

        let mut ctx = Context::for_function(&params, return_ty, None, None, None, false, true);
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
        sa.analyze_stmts(body, &mut ctx);
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

        // Inferred return type → Salsa, via the priming-sweep buffer.
        if let Some(fqn) = fqn {
            self.record_function_inference(&fqn, &inferred);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_class_decl_typed<'arena, 'src>(
        &self,
        decl: &php_ast::ast::ClassDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let class_name = decl.name.unwrap_or("<anonymous>");
        let resolved = resolve_name_via_db(self.db, file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let parent_fqcn = self
            .db
            .lookup_class_node(fqcn)
            .and_then(|node| node.parent(self.db));

        if let Some(parent) = &decl.extends {
            check_name_class(parent, self.db, file, source, source_map, all_issues);
        }
        for iface in decl.implements.iter() {
            check_name_class(iface, self.db, file, source, source_map, all_issues);
        }

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = crate::db::lookup_method_in_chain(self.db, fqcn, method.name)
                .map(|n| (n.params(self.db).to_vec(), n.return_type(self.db)))
                .unwrap_or_default();

            let is_ctor = method.name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
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
                    method: Arc::from(method.name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );

            emit_unused_params(&params, &ctx, method.name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method.name, &inferred);
        }

        self.check_trait_constraints(fqcn, file, all_issues);
    }

    /// Emit `InvalidTraitUse` issues if this class violates any `@psalm-require-extends` /
    /// `@psalm-require-implements` constraint declared on the traits it uses.
    fn check_trait_constraints(&self, fqcn: &str, file: &Arc<str>, all_issues: &mut Vec<Issue>) {
        // Used-trait list, ancestor chain, and `@psalm-require-*` constraints
        // all come from the salsa db.
        let Some(node) = self.db.lookup_class_node(fqcn) else {
            return;
        };
        let trait_list = node.traits(self.db);
        let class_all_parents: Vec<Arc<str>> = crate::db::class_ancestors(self.db, node).0;

        for trait_fqcn in trait_list.iter() {
            let Some(trait_node) = self
                .db
                .lookup_class_node(trait_fqcn.as_ref())
                .filter(|n| n.active(self.db))
            else {
                continue;
            };
            let req_ext = trait_node.require_extends(self.db);
            let req_impl = trait_node.require_implements(self.db);
            if req_ext.is_empty() && req_impl.is_empty() {
                continue;
            }
            // Derive short name from the FQCN's trailing segment.  Matches
            // what the collector stores in `TraitStorage::short_name`.
            let tr_short: Arc<str> = trait_fqcn
                .rsplit('\\')
                .next()
                .map(Arc::from)
                .unwrap_or_else(|| trait_fqcn.clone());

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
                        mir_issues::Location {
                            file: file.clone(),
                            line: 1,
                            line_end: 1,
                            col_start: 0,
                            col_end: 0,
                        },
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
                        mir_issues::Location {
                            file: file.clone(),
                            line: 1,
                            line_end: 1,
                            col_start: 0,
                            col_end: 0,
                        },
                    ));
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_trait_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::TraitDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved = resolve_name_via_db(self.db, file.as_ref(), decl.name);
        let fqcn: &str = &resolved;

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = crate::db::lookup_method_in_chain(self.db, fqcn, method.name)
                .map(|n| (n.params(self.db).to_vec(), n.return_type(self.db)))
                .unwrap_or_default();

            let is_ctor = method.name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
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

            emit_unused_params(&params, &ctx, method.name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method.name, &inferred);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn analyze_trait_decl_typed<'arena, 'src>(
        &self,
        decl: &php_ast::ast::TraitDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
        type_envs: &mut std::collections::HashMap<
            crate::type_env::ScopeId,
            crate::type_env::TypeEnv,
        >,
        all_symbols: &mut Vec<ResolvedSymbol>,
    ) {
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved = resolve_name_via_db(self.db, file.as_ref(), decl.name);
        let fqcn: &str = &resolved;

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = crate::db::lookup_method_in_chain(self.db, fqcn, method.name)
                .map(|n| (n.params(self.db).to_vec(), n.return_type(self.db)))
                .unwrap_or_default();

            let is_ctor = method.name == "__construct";
            let mut ctx = Context::for_method(
                &params,
                return_ty,
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
                    method: Arc::from(method.name),
                },
                crate::type_env::TypeEnv::new(ctx.vars.clone()),
            );

            emit_unused_params(&params, &ctx, method.name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            self.record_method_inference(fqcn, method.name, &inferred);
        }
    }

    fn analyze_enum_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::EnumDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        use php_ast::ast::EnumMemberKind;
        for iface in decl.implements.iter() {
            check_name_class(iface, self.db, file, source, source_map, all_issues);
        }
        for member in decl.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }
        }
    }

    fn analyze_interface_decl<'arena, 'src>(
        &self,
        decl: &php_ast::ast::InterfaceDecl<'arena, 'src>,
        file: &Arc<str>,
        source: &str,
        source_map: &php_rs_parser::source_map::SourceMap,
        all_issues: &mut Vec<Issue>,
    ) {
        use php_ast::ast::ClassMemberKind;
        for parent in decl.extends.iter() {
            check_name_class(parent, self.db, file, source, source_map, all_issues);
        }
        for member in decl.members.iter() {
            let ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.db, file, source, source_map, all_issues);
            }
        }
    }
}

// ---------------------------------------------------------------------------

/// Seed `ctx.var_locations` for function/method parameters using their AST spans.
fn seed_param_locations(
    ctx: &mut crate::context::Context,
    ast_params: &php_ast::ast::ArenaVec<'_, php_ast::ast::Param<'_, '_>>,
    source: &str,
    source_map: &php_rs_parser::source_map::SourceMap,
) {
    for p in ast_params.iter() {
        let name = p.name.trim_start_matches('$');
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
    return_types
        .iter()
        .fold(Union::empty(), |acc, t| Union::merge(&acc, t))
}
