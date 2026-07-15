use super::StatementsAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{Issue, IssueKind, Location};
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
        // Parameter defaults are constant expressions outside the body flow;
        // analyze them so `Cfg::MODE`-style defaults record references.
        for p in decl.params.iter() {
            if let Some(default) = &p.default {
                let mut default_ctx = ctx.clone();
                let mut ea = self.expr_analyzer(&default_ctx);
                let _ = ea.analyze(default, &mut default_ctx);
            }
        }
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
        sa.collect_symbols = self.collect_symbols;
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

        // Anonymous classes (and named classes declared nested inside a
        // function/if-block) are never collected into the codebase's class
        // definitions, so unlike a top-level `class Foo extends Bar {}` —
        // checked once from the collected definition — nothing else ever
        // validates their `extends`/`implements`/`use` targets. Each name is
        // resolved up front to honor `class_exists`/`interface_exists`/
        // `trait_exists` guards, matching the top-level check's behavior.
        if let Some(parent) = &decl.extends {
            let parent_str = crate::parser::name_to_string_owned(parent);
            let parent_resolved = crate::db::resolve_name(self.db, &self.file, &parent_str);
            if !ctx.is_class_guarded(parent_resolved.as_str()) {
                self.check_name_undefined_class(parent);
            }
            self.record_class_like_ref(&parent_resolved, parent.span);
            self.check_extends_final_class(&parent_resolved, &fqcn, parent.span);
        }
        for iface in decl.implements.iter() {
            let iface_str = crate::parser::name_to_string_owned(iface);
            let iface_resolved = crate::db::resolve_name(self.db, &self.file, &iface_str);
            if !ctx.is_class_guarded(iface_resolved.as_str()) {
                self.check_name_undefined_class(iface);
            }
            self.record_class_like_ref(&iface_resolved, iface.span);
        }
        for member in decl.body.members.iter() {
            if let ClassMemberKind::TraitUse(tu) = &member.kind {
                for trait_name in tu.traits.iter() {
                    let trait_str = crate::parser::name_to_string_owned(trait_name);
                    let trait_resolved = crate::db::resolve_name(self.db, &self.file, &trait_str);
                    if !ctx.is_class_guarded(trait_resolved.as_str()) {
                        self.check_name_undefined_trait(trait_name);
                    }
                    self.record_class_like_ref(&trait_resolved, trait_name.span);
                }
            }
        }

        for member in decl.body.members.iter() {
            // Property initializers are constant expressions outside any body
            // flow; analyze them so `Widget::class`-style defaults record
            // class/constant references.
            if let ClassMemberKind::Property(prop) = &member.kind {
                if let Some(default) = &prop.default {
                    let mut ea = self.expr_analyzer(&param_default_ctx);
                    let _ = ea.analyze(default, &mut param_default_ctx);
                }
                continue;
            }
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
            sa.collect_symbols = self.collect_symbols;
            sa.analyze_stmts(&body.stmts, &mut method_ctx);
        }
    }

    /// Emit `InvalidExtendClass` if `parent_resolved` names a `final` class.
    /// Only for anonymous/nested classes (see `analyze_class_decl_stmt`) — a
    /// top-level class's `extends` is checked once via `ClassAnalyzer::analyze_all`
    /// (`class/mod.rs`), which never sees anonymous/nested classes since the
    /// collector skips them entirely.
    fn check_extends_final_class(&mut self, parent_resolved: &str, child_fqcn: &str, span: Span) {
        let here = crate::db::Fqcn::from_str(self.db, parent_resolved);
        let Some(parent_class) = crate::db::find_class_like(self.db, here) else {
            return;
        };
        if !parent_class.is_final() {
            return;
        }
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (line_end, col_end) = self.offset_to_line_col(span.end);
        self.issues.add(Issue::new(
            IssueKind::InvalidExtendClass {
                parent: parent_resolved.to_string(),
                child: child_fqcn.to_string(),
            },
            Location {
                file: self.file.clone(),
                line,
                line_end,
                col_start,
                col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
            },
        ));
    }
}
