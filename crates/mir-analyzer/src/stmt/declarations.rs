use super::StatementsAnalyzer;
use crate::flow_state::FlowState;
use mir_types::Name;
use php_ast::owned::{ClassDecl, ClassMemberKind, FunctionDecl, Param};
use php_ast::Span;
use std::sync::Arc;

/// Return the tight byte-offset span covering only the `$name` token within a
/// parameter declaration, falling back to the full param span when not found.
fn param_name_span(source: &str, p: &Param) -> Span {
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
        Span {
            start,
            end: start + needle.len() as u32,
        }
    } else {
        p.span
    }
}

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
            |fqn: &str| -> Option<(Vec<mir_codebase::DeclaredParam>, Option<mir_types::Type>)> {
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
                let ast_params: Vec<mir_codebase::DeclaredParam> = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::DeclaredParam {
                        name: Name::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
                        ty: None,
                        out_ty: None,
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
                let ast_params: Vec<mir_codebase::DeclaredParam> = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::DeclaredParam {
                        name: Name::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
                        ty: None,
                        out_ty: None,
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
        for p in decl.params.iter() {
            if let Some(raw) = p.name.as_deref() {
                let trimmed = raw.trim_start_matches('$');
                let ty = fn_ctx.get_var(trimmed);
                self.record_symbol_for_var(param_name_span(self.source, p), trimmed, ty);
            }
        }
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
        // Anonymous classes are not collected into the DB (the collector skips
        // them), so `find_class_like` always returns None for them. Derive the
        // parent FQCN directly from the AST extends clause instead.
        let parent_fqcn = if decl.name.as_ref().and_then(|i| i.as_deref()).is_some() {
            crate::db::find_class_like(self.db, here).and_then(|c| c.parent().cloned())
        } else {
            decl.extends.as_ref().map(|parent_name| {
                let parent_str = crate::parser::name_to_string_owned(parent_name);
                let resolved_parent = crate::db::resolve_name(self.db, &self.file, &parent_str);
                Arc::from(resolved_parent.as_str())
            })
        };

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
                    .map(|p| mir_codebase::DeclaredParam {
                        name: Name::new(p.name.as_deref().unwrap_or("").trim_start_matches('$')),
                        ty: None,
                        out_ty: None,
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
            method_ctx.current_method_name = Some(Arc::from(method_name));
            for p in method.params.iter() {
                if let Some(raw) = p.name.as_deref() {
                    let trimmed = raw.trim_start_matches('$');
                    let ty = method_ctx.get_var(trimmed);
                    self.record_symbol_for_var(param_name_span(self.source, p), trimmed, ty);
                }
            }
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
