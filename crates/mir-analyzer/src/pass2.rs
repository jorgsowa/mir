use std::sync::Arc;

use mir_codebase::Codebase;
use mir_issues::Issue;
use mir_types::Union;

use crate::diagnostics::{
    check_name_class, check_type_hint_classes, emit_unused_params, emit_unused_variables,
};
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

// ---------------------------------------------------------------------------
// Pass2Driver
// ---------------------------------------------------------------------------

pub(crate) struct Pass2Driver<'a> {
    codebase: &'a Arc<Codebase>,
    php_version: PhpVersion,
}

impl<'a> Pass2Driver<'a> {
    pub(crate) fn new(codebase: &'a Arc<Codebase>, php_version: PhpVersion) -> Self {
        Self {
            codebase,
            php_version,
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

        // Analyze top-level executable statements in global scope.
        {
            use crate::context::Context;
            use crate::stmt::StatementsAnalyzer;
            use mir_issues::IssueBuffer;

            let mut ctx = Context::new();
            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.codebase,
                file.clone(),
                source,
                source_map,
                &mut buf,
                &mut all_symbols,
                self.php_version,
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
                self.codebase,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
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
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
        }
        use crate::context::Context;
        use crate::stmt::StatementsAnalyzer;
        use mir_issues::IssueBuffer;

        let resolved_fn = self.codebase.resolve_class_name(file.as_ref(), fn_name);
        let func_opt: Option<mir_codebase::storage::FunctionStorage> = self
            .codebase
            .functions
            .get(resolved_fn.as_str())
            .map(|r| r.clone())
            .or_else(|| self.codebase.functions.get(fn_name).map(|r| r.clone()))
            .or_else(|| {
                self.codebase
                    .functions
                    .iter()
                    .find(|e| e.short_name.as_ref() == fn_name)
                    .map(|e| e.value().clone())
            });

        let fqn = func_opt.as_ref().map(|f| f.fqn.clone());
        let (params, return_ty): (Vec<mir_codebase::FnParam>, _) = match &func_opt {
            Some(f)
                if f.params.len() == decl.params.len()
                    && f.params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| cp.name.as_ref() == ap.name) =>
            {
                (f.params.clone(), f.return_type.clone())
            }
            _ => {
                let ast_params = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Arc::from(p.name),
                        ty: None,
                        default: p.default.as_ref().map(|_| mir_types::Union::mixed()),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                (ast_params, None)
            }
        };

        let mut ctx = Context::for_function(&params, return_ty, None, None, None, false, true);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.codebase,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
        );
        sa.analyze_stmts(body, &mut ctx);
        let inferred = merge_return_types(&sa.return_types);
        drop(sa);

        emit_unused_params(&params, &ctx, "", file, all_issues);
        emit_unused_variables(&ctx, file, all_issues);
        all_issues.extend(buf.into_issues());

        if let Some(fqn) = fqn {
            if let Some(mut func) = self.codebase.functions.get_mut(fqn.as_ref()) {
                func.inferred_return_type = Some(inferred);
            }
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
        let resolved = self.codebase.resolve_class_name(file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let parent_fqcn = self
            .codebase
            .classes
            .get(fqcn)
            .and_then(|c| c.parent.clone());

        if let Some(parent) = &decl.extends {
            check_name_class(parent, self.codebase, file, source, source_map, all_issues);
        }
        for iface in decl.implements.iter() {
            check_name_class(iface, self.codebase, file, source, source_map, all_issues);
        }

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = self
                .codebase
                .get_method(fqcn, method.name)
                .as_deref()
                .map(|m| (m.params.clone(), m.return_type.clone()))
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

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.codebase,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
            );
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            emit_unused_params(&params, &ctx, method.name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            if let Some(mut cls) = self.codebase.classes.get_mut(fqcn) {
                if let Some(m) = cls.own_methods.get_mut(method.name) {
                    Arc::make_mut(m).inferred_return_type = Some(inferred);
                }
            }
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
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }
        }
        if let Some(hint) = &decl.return_type {
            check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
        }

        let resolved_fn = self.codebase.resolve_class_name(file.as_ref(), fn_name);
        let func_opt: Option<mir_codebase::storage::FunctionStorage> = self
            .codebase
            .functions
            .get(resolved_fn.as_str())
            .map(|r| r.clone())
            .or_else(|| self.codebase.functions.get(fn_name).map(|r| r.clone()))
            .or_else(|| {
                self.codebase
                    .functions
                    .iter()
                    .find(|e| e.short_name.as_ref() == fn_name)
                    .map(|e| e.value().clone())
            });

        let fqn = func_opt.as_ref().map(|f| f.fqn.clone());
        let (params, return_ty): (Vec<mir_codebase::FnParam>, _) = match &func_opt {
            Some(f)
                if f.params.len() == decl.params.len()
                    && f.params
                        .iter()
                        .zip(decl.params.iter())
                        .all(|(cp, ap)| cp.name.as_ref() == ap.name) =>
            {
                (f.params.clone(), f.return_type.clone())
            }
            _ => {
                let ast_params = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Arc::from(p.name),
                        ty: None,
                        default: p.default.as_ref().map(|_| mir_types::Union::mixed()),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                (ast_params, None)
            }
        };

        let mut ctx = Context::for_function(&params, return_ty, None, None, None, false, true);
        let mut buf = IssueBuffer::new();
        let mut sa = StatementsAnalyzer::new(
            self.codebase,
            file.clone(),
            source,
            source_map,
            &mut buf,
            all_symbols,
            self.php_version,
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

        if let Some(fqn) = fqn {
            if let Some(mut func) = self.codebase.functions.get_mut(fqn.as_ref()) {
                func.inferred_return_type = Some(inferred);
            }
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
        let resolved = self.codebase.resolve_class_name(file.as_ref(), class_name);
        let fqcn: &str = &resolved;
        let parent_fqcn = self
            .codebase
            .classes
            .get(fqcn)
            .and_then(|c| c.parent.clone());

        if let Some(parent) = &decl.extends {
            check_name_class(parent, self.codebase, file, source, source_map, all_issues);
        }
        for iface in decl.implements.iter() {
            check_name_class(iface, self.codebase, file, source, source_map, all_issues);
        }

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = self
                .codebase
                .get_method(fqcn, method.name)
                .as_deref()
                .map(|m| (m.params.clone(), m.return_type.clone()))
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

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.codebase,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
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

            if let Some(mut cls) = self.codebase.classes.get_mut(fqcn) {
                if let Some(m) = cls.own_methods.get_mut(method.name) {
                    Arc::make_mut(m).inferred_return_type = Some(inferred);
                }
            }
        }

        self.check_trait_constraints(fqcn, file, all_issues);
    }

    /// Emit `InvalidTraitUse` issues if this class violates any `@psalm-require-extends` /
    /// `@psalm-require-implements` constraint declared on the traits it uses.
    fn check_trait_constraints(&self, fqcn: &str, file: &Arc<str>, all_issues: &mut Vec<Issue>) {
        // Check @psalm-require-extends / @psalm-require-implements for each used trait.
        let (class_all_parents, trait_list) = self
            .codebase
            .classes
            .get(fqcn)
            .map(|c| (c.all_parents.clone(), c.traits.clone()))
            .unwrap_or_default();

        for trait_fqcn in &trait_list {
            let Some(tr) = self.codebase.traits.get(trait_fqcn.as_ref()) else {
                continue;
            };
            let req_ext = tr.require_extends.clone();
            let req_impl = tr.require_implements.clone();
            let tr_short = tr.short_name.clone();
            drop(tr);

            for req in &req_ext {
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

            for req in &req_impl {
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

        let resolved = self.codebase.resolve_class_name(file.as_ref(), decl.name);
        let fqcn: &str = &resolved;

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = self
                .codebase
                .get_method(fqcn, method.name)
                .as_deref()
                .map(|m| (m.params.clone(), m.return_type.clone()))
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

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.codebase,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
            );
            sa.analyze_stmts(body, &mut ctx);
            let inferred = merge_return_types(&sa.return_types);
            drop(sa);

            emit_unused_params(&params, &ctx, method.name, file, all_issues);
            emit_unused_variables(&ctx, file, all_issues);
            all_issues.extend(buf.into_issues());

            if let Some(mut tr) = self.codebase.traits.get_mut(fqcn) {
                if let Some(m) = tr.own_methods.get_mut(method.name) {
                    Arc::make_mut(m).inferred_return_type = Some(inferred);
                }
            }
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

        let resolved = self.codebase.resolve_class_name(file.as_ref(), decl.name);
        let fqcn: &str = &resolved;

        for member in decl.members.iter() {
            if let php_ast::ast::ClassMemberKind::Property(prop) = &member.kind {
                if let Some(hint) = &prop.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
                continue;
            }
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };

            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }

            let Some(body) = &method.body else { continue };

            let (params, return_ty) = self
                .codebase
                .get_method(fqcn, method.name)
                .as_deref()
                .map(|m| (m.params.clone(), m.return_type.clone()))
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

            let mut buf = IssueBuffer::new();
            let mut sa = StatementsAnalyzer::new(
                self.codebase,
                file.clone(),
                source,
                source_map,
                &mut buf,
                all_symbols,
                self.php_version,
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

            if let Some(mut tr) = self.codebase.traits.get_mut(fqcn) {
                if let Some(m) = tr.own_methods.get_mut(method.name) {
                    Arc::make_mut(m).inferred_return_type = Some(inferred);
                }
            }
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
            check_name_class(iface, self.codebase, file, source, source_map, all_issues);
        }
        for member in decl.members.iter() {
            let EnumMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
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
            check_name_class(parent, self.codebase, file, source, source_map, all_issues);
        }
        for member in decl.members.iter() {
            let ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for param in method.params.iter() {
                if let Some(hint) = &param.type_hint {
                    check_type_hint_classes(
                        hint,
                        self.codebase,
                        file,
                        source,
                        source_map,
                        all_issues,
                    );
                }
            }
            if let Some(hint) = &method.return_type {
                check_type_hint_classes(hint, self.codebase, file, source, source_map, all_issues);
            }
        }
    }
}

// ---------------------------------------------------------------------------

pub fn merge_return_types(return_types: &[Union]) -> Union {
    if return_types.is_empty() {
        return Union::single(mir_types::Atomic::TVoid);
    }
    return_types
        .iter()
        .fold(Union::empty(), |acc, t| Union::merge(&acc, t))
}
