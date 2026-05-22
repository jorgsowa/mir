use super::StatementsAnalyzer;
use crate::context::Context;
use mir_types::Symbol;
use php_ast::owned::{ClassDecl, ClassMemberKind, FunctionDecl};
use std::sync::Arc;

impl<'a> StatementsAnalyzer<'a> {
    pub(crate) fn analyze_function_decl_stmt(&mut self, decl: &FunctionDecl, ctx: &mut Context) {
        for p in decl.params.iter() {
            if let Some(default) = &p.default {
                let mut ea = self.expr_analyzer(ctx);
                let _ = ea.analyze(default, ctx);
            }
        }

        // Look up the function in the database to get resolved parameter types
        let fn_name = decl.name.as_deref().unwrap_or("").to_string();
        let resolve_fn =
            |fqn: &str| -> Option<(Vec<mir_codebase::FnParam>, Option<mir_types::Union>)> {
                let db = self.db;
                let here = crate::db::Fqcn::new(db, Arc::<str>::from(fqn));
                crate::db::find_function(db, here).map(|f| {
                    (
                        f.params.to_vec(),
                        f.return_type.as_ref().map(|t| (**t).clone()),
                    )
                })
            };
        let (params, return_ty) = if let Some(ns) = self.db.file_namespace(&self.file) {
            let fqn = format!("{}\\{}", ns, fn_name);
            if let Some(found) = resolve_fn(&fqn).or_else(|| resolve_fn(&fn_name)) {
                found
            } else {
                let ast_params: Vec<mir_codebase::FnParam> = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Symbol::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
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
            if let Some(found) = resolve_fn(&fn_name) {
                found
            } else {
                let ast_params: Vec<mir_codebase::FnParam> = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Symbol::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
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

    pub(crate) fn analyze_class_decl_stmt(&mut self, decl: &ClassDecl, ctx: &mut Context) {
        let class_name = decl
            .name
            .as_ref()
            .and_then(|i| i.as_deref())
            .unwrap_or("<anonymous>");
        let resolved = crate::db::resolve_name_via_db(self.db, &self.file, class_name);
        let fqcn: Arc<str> = Arc::from(resolved.as_str());
        let here = crate::db::Fqcn::new(self.db, fqcn.clone());
        let parent_fqcn =
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

        let mut param_default_ctx = ctx.clone();
        param_default_ctx.self_fqcn = Some(fqcn.clone());
        param_default_ctx.parent_fqcn = parent_fqcn.clone();
        param_default_ctx.static_fqcn = Some(fqcn.clone());

        for member in decl.members.iter() {
            let ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            for p in method.params.iter() {
                if let Some(default) = &p.default {
                    let mut ea = self.expr_analyzer(&param_default_ctx);
                    let _ = ea.analyze(default, &mut param_default_ctx);
                }
            }
            let Some(body) = &method.body else { continue };
            let method_name = method.name.as_deref().unwrap_or("");
            let pulled = crate::db::find_method_in_chain(
                self.db,
                crate::db::Fqcn::new(self.db, fqcn.clone()),
                method_name,
            );
            let (params, return_ty) = if let Some((_, storage)) = pulled {
                (
                    storage.params.to_vec(),
                    storage.return_type.as_deref().cloned(),
                )
            } else {
                let ast_params = method
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: Symbol::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
                        ty: None,
                        has_default: p.default.is_some(),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                (ast_params, None)
            };
            let is_ctor = method_name == "__construct";
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
