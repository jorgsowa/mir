use super::StatementsAnalyzer;
use crate::context::Context;
use std::sync::Arc;

impl<'a> StatementsAnalyzer<'a> {
    pub(super) fn analyze_function_decl_stmt<'arena, 'src>(
        &mut self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        ctx: &mut Context,
    ) {
        for p in decl.params.iter() {
            if let Some(default) = &p.default {
                let mut ea = self.expr_analyzer(ctx);
                let _ = ea.analyze(default, ctx);
            }
        }

        // Look up the function in the database to get resolved parameter types
        let fn_name = decl.name.to_string();
        let (params, return_ty) = if let Some(ns) = self.db.file_namespace(&self.file) {
            let fqn = format!("{}\\{}", ns, fn_name);
            if let Some(node) = self
                .db
                .lookup_function_node(&fqn)
                .filter(|n| n.active(self.db))
            {
                (
                    node.params(self.db).to_vec(),
                    node.return_type(self.db).map(|t| (*t).clone()),
                )
            } else {
                // Try global namespace
                if let Some(node) = self
                    .db
                    .lookup_function_node(&fn_name)
                    .filter(|n| n.active(self.db))
                {
                    (
                        node.params(self.db).to_vec(),
                        node.return_type(self.db).map(|t| (*t).clone()),
                    )
                } else {
                    // Fallback to AST if not found in database
                    let ast_params: Vec<mir_codebase::FnParam> = decl
                        .params
                        .iter()
                        .map(|p| mir_codebase::FnParam {
                            name: Arc::from(p.name.to_string().trim_start_matches('$')),
                            ty: None,
                            has_default: p.default.is_some(),
                            is_variadic: p.variadic,
                            is_byref: p.by_ref,
                            is_optional: p.default.is_some() || p.variadic,
                        })
                        .collect();
                    (ast_params, None)
                }
            }
        } else {
            // No namespace
            if let Some(node) = self
                .db
                .lookup_function_node(&fn_name)
                .filter(|n| n.active(self.db))
            {
                (
                    node.params(self.db).to_vec(),
                    node.return_type(self.db).map(|t| (*t).clone()),
                )
            } else {
                // Fallback to AST if not found in database
                let ast_params: Vec<mir_codebase::FnParam> = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Arc::from(p.name.to_string().trim_start_matches('$')),
                        ty: None,
                        has_default: p.default.is_some(),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                (ast_params, None)
            }
        };

        let mut fn_ctx = Context::for_function(
            &params,
            return_ty,
            Arc::from([]),
            None,
            None,
            None,
            ctx.strict_types,
            true,
        );
        let mut sa = StatementsAnalyzer::new(
            self.db,
            self.file.clone(),
            self.source,
            self.source_map,
            self.issues,
            self.symbols,
            self.php_version,
            self.inference_only,
        );
        sa.analyze_stmts(&decl.body, &mut fn_ctx);
    }

    pub(super) fn analyze_class_decl_stmt<'arena, 'src>(
        &mut self,
        decl: &php_ast::ast::ClassDecl<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let class_name_owned = decl
            .name
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<anonymous>".to_string());
        let class_name = class_name_owned.as_str();
        let resolved = crate::db::resolve_name_via_db(self.db, &self.file, class_name);
        let fqcn: Arc<str> = Arc::from(resolved.as_str());
        let parent_fqcn = self
            .db
            .lookup_class_node(fqcn.as_ref())
            .and_then(|node| node.parent(self.db));

        let mut param_default_ctx = ctx.clone();
        param_default_ctx.self_fqcn = Some(fqcn.clone());
        param_default_ctx.parent_fqcn = parent_fqcn.clone();
        param_default_ctx.static_fqcn = Some(fqcn.clone());

        for member in decl.members.iter() {
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for p in method.params.iter() {
                if let Some(default) = &p.default {
                    let mut ea = self.expr_analyzer(&param_default_ctx);
                    let _ = ea.analyze(default, &mut param_default_ctx);
                }
            }
            let Some(body) = &method.body else { continue };
            let (params, return_ty) =
                crate::db::lookup_method_in_chain(self.db, fqcn.as_ref(), &method.name.to_string())
                    .map(|n| {
                        (
                            n.params(self.db).to_vec(),
                            n.return_type(self.db).map(|t| (*t).clone()),
                        )
                    })
                    .unwrap_or_else(|| {
                        let ast_params = method
                            .params
                            .iter()
                            .map(|p| mir_codebase::FnParam {
                                name: Arc::from(p.name.to_string().trim_start_matches('$')),
                                ty: None,
                                has_default: p.default.is_some(),
                                is_variadic: p.variadic,
                                is_byref: p.by_ref,
                                is_optional: p.default.is_some() || p.variadic,
                            })
                            .collect();
                        (ast_params, None)
                    });
            let is_ctor = method.name == "__construct";
            let mut method_ctx = Context::for_method(
                &params,
                return_ty,
                Arc::from([]),
                Some(fqcn.clone()),
                parent_fqcn.clone(),
                Some(fqcn.clone()),
                ctx.strict_types,
                is_ctor,
                method.is_static,
            );
            let mut sa = StatementsAnalyzer::new(
                self.db,
                self.file.clone(),
                self.source,
                self.source_map,
                self.issues,
                self.symbols,
                self.php_version,
                self.inference_only,
            );
            sa.analyze_stmts(body, &mut method_ctx);
        }
    }
}
