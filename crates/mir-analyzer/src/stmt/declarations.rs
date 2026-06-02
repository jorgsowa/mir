use super::StatementsAnalyzer;
use crate::flow_state::FlowState;
use mir_types::Name;
use php_ast::owned::{ClassDecl, ClassMemberKind, FunctionDecl};
use std::sync::Arc;

impl<'a> StatementsAnalyzer<'a> {
    pub(crate) fn analyze_function_decl_stmt(&mut self, decl: &FunctionDecl, ctx: &mut FlowState) {
        for p in decl.params.iter() {
            if let Some(default) = &p.default {
                let mut ea = self.expr_analyzer(ctx);
                let _ = ea.analyze(default, ctx);
            }
        }

        // Look up the function in the database to get resolved parameter types
        let fn_name = decl.name.as_deref().unwrap_or("").to_string();
        let resolve_fn =
            |fqn: &str| -> Option<(Vec<mir_codebase::FnParam>, Option<mir_types::Type>)> {
                let db = self.db;
                let here = crate::db::Fqcn::from_str(db, fqn);
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
                        name: Name::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
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
                        name: Name::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
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

        let mut fn_ctx = FlowState::for_function(
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
            self.mode,
        );
        sa.analyze_stmts(&decl.body.stmts, &mut fn_ctx);
    }

    pub(crate) fn analyze_class_decl_stmt(&mut self, decl: &ClassDecl, ctx: &mut FlowState) {
        let class_name = decl
            .name
            .as_ref()
            .and_then(|i| i.as_deref())
            .unwrap_or("<anonymous>");
        let resolved = crate::db::resolve_name(self.db, &self.file, class_name);
        let fqcn: Arc<str> = Arc::from(resolved.as_str());
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        let parent_fqcn =
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned());

        let mut param_default_ctx = ctx.clone();
        param_default_ctx.self_fqcn = Some(fqcn.clone());
        param_default_ctx.parent_fqcn = parent_fqcn.clone();
        param_default_ctx.static_fqcn = Some(fqcn.clone());

        for member in decl.body.members.iter() {
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
                crate::db::Fqcn::from_str(self.db, fqcn.as_ref()),
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
                        name: Name::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
                        ty: None,
                        has_default: p.default.is_some(),
                        is_variadic: p.variadic,
                        is_byref: p.by_ref,
                        is_optional: p.default.is_some() || p.variadic,
                    })
                    .collect();
                // Anonymous classes are not collected into storage, so derive
                // the declared return type straight from the AST hint — without
                // this, return-type checks (e.g. `return 5` from a `: string`
                // method) are silently skipped for anonymous-class methods.
                let ret = method
                    .return_type
                    .as_ref()
                    .map(|h| crate::parser::type_from_hint_owned(h, Some(fqcn.as_ref())));
                (ast_params, ret)
            };
            let is_ctor = method_name == "__construct";
            let mut method_ctx = FlowState::for_method(
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
                self.mode,
            );
            sa.analyze_stmts(&body.stmts, &mut method_ctx);
        }
    }
}
