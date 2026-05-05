use super::StatementsAnalyzer;
use crate::context::Context;
use mir_types::Union;
use std::sync::Arc;

impl<'a> StatementsAnalyzer<'a> {
    pub(super) fn analyze_function_decl_stmt<'arena, 'src>(
        &mut self,
        decl: &php_ast::ast::FunctionDecl<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let params: Vec<mir_codebase::FnParam> = decl
            .params
            .iter()
            .map(|p| mir_codebase::FnParam {
                name: Arc::from(p.name.trim_start_matches('$')),
                ty: None,
                default: p.default.as_ref().map(|_| Union::mixed()),
                is_variadic: p.variadic,
                is_byref: p.by_ref,
                is_optional: p.default.is_some() || p.variadic,
            })
            .collect();
        let mut fn_ctx =
            Context::for_function(&params, None, None, None, None, ctx.strict_types, true);
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
        let class_name = decl.name.unwrap_or("<anonymous>");
        let resolved = crate::db::resolve_name_via_db(self.db, &self.file, class_name);
        let fqcn: Arc<str> = Arc::from(resolved.as_str());
        let parent_fqcn = self
            .db
            .lookup_class_node(fqcn.as_ref())
            .and_then(|node| node.parent(self.db));

        for member in decl.members.iter() {
            let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            let Some(body) = &method.body else { continue };
            let (params, return_ty) =
                crate::db::lookup_method_in_chain(self.db, fqcn.as_ref(), method.name)
                    .map(|n| (n.params(self.db).to_vec(), n.return_type(self.db)))
                    .unwrap_or_else(|| {
                        let ast_params = method
                            .params
                            .iter()
                            .map(|p| mir_codebase::FnParam {
                                name: p.name.trim_start_matches('$').into(),
                                ty: None,
                                default: p.default.as_ref().map(|_| mir_types::Union::mixed()),
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
