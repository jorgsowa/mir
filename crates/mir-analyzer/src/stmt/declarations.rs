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
        // Phase 4: pull path (find_function) → fallback to push-path.
        let resolve_fn =
            |fqn: &str| -> Option<(Vec<mir_codebase::FnParam>, Option<mir_types::Union>)> {
                let db = self.db;
                let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqn));
                if let Some(f) = crate::db::find_function(db, here) {
                    return Some((
                        f.params.to_vec(),
                        f.return_type.as_ref().map(|t| (**t).clone()),
                    ));
                }
                db.lookup_function_node(fqn)
                    .filter(|n| n.active(db))
                    .map(|node| {
                        (
                            node.params(db).to_vec(),
                            node.return_type(db).map(|t| (*t).clone()),
                        )
                    })
            };
        let (params, return_ty) = if let Some(ns) = self.db.file_namespace(&self.file) {
            let fqn = format!("{}\\{}", ns, fn_name);
            if let Some(found) = resolve_fn(&fqn).or_else(|| resolve_fn(&fn_name)) {
                found
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
        } else {
            // No namespace
            if let Some(found) = resolve_fn(&fn_name) {
                found
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
        // Phase 4: pull path first; push-path fallback for tests.
        let here = crate::db::Fqcn::new(self.db, fqcn.clone());
        let parent_fqcn = crate::db::find_class_like(self.db, here)
            .and_then(|c| c.parent().cloned())
            .or_else(|| {
                self.db
                    .lookup_class_node(fqcn.as_ref())
                    .and_then(|node| node.parent(self.db))
            });

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
            let pulled = crate::db::find_method_in_chain(
                self.db,
                crate::db::Fqcn::new(self.db, fqcn.clone()),
                &method.name.to_string(),
            );
            let (params, return_ty) = if let Some((_, storage)) = pulled {
                (
                    storage.params.to_vec(),
                    storage.return_type.as_deref().cloned(),
                )
            } else if let Some(n) =
                crate::db::lookup_method_in_chain(self.db, fqcn.as_ref(), &method.name.to_string())
            {
                (
                    n.params(self.db).to_vec(),
                    n.return_type(self.db).map(|t| (*t).clone()),
                )
            } else {
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
            };
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
