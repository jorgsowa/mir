use super::helpers::{ast_params_to_fn_params_resolved, resolve_named_objects_in_union};
use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_types::{Atomic, Union};
use php_ast::ast::{ArrowFunctionExpr, ClosureExpr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_closure<'arena, 'src>(
        &mut self,
        c: &ClosureExpr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        for param in c.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_type_hint(hint);
            }
        }
        if let Some(hint) = &c.return_type {
            self.check_type_hint(hint);
        }

        let params = ast_params_to_fn_params_resolved(
            &c.params,
            ctx.self_fqcn.as_deref(),
            self.db,
            &self.file,
        );
        let return_ty_hint = c
            .return_type
            .as_ref()
            .map(|h| crate::parser::type_from_hint(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union(u, self.db, &self.file));

        let mut closure_ctx = crate::context::Context::for_function(
            &params,
            return_ty_hint.clone(),
            ctx.self_fqcn.clone(),
            ctx.parent_fqcn.clone(),
            ctx.static_fqcn.clone(),
            ctx.strict_types,
            c.is_static,
        );
        for use_var in c.use_vars.iter() {
            let name = use_var.name.trim_start_matches('$');
            closure_ctx.set_var(name, ctx.get_var(name));
            if ctx.is_tainted(name) {
                closure_ctx.taint_var(name);
            }
        }

        let inferred_return = {
            let mut sa = crate::stmt::StatementsAnalyzer::new(
                self.db,
                self.file.clone(),
                self.source,
                self.source_map,
                self.issues,
                self.symbols,
                self.php_version,
                self.inference_only,
            );
            sa.analyze_stmts(&c.body, &mut closure_ctx);
            let ret = crate::project::merge_return_types(&sa.return_types);
            drop(sa);
            ret
        };

        for name in &closure_ctx.read_vars {
            ctx.read_vars.insert(name.clone());
        }

        let return_ty = return_ty_hint.unwrap_or(inferred_return);
        let closure_params: Vec<mir_types::atomic::FnParam> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: p.name.clone(),
                ty: p.ty.clone(),
                default: p.default.clone(),
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();

        Union::single(Atomic::TClosure {
            params: closure_params,
            return_type: Box::new(return_ty),
            this_type: ctx.self_fqcn.clone().map(|f| {
                Box::new(Union::single(Atomic::TNamedObject {
                    fqcn: f,
                    type_params: vec![],
                }))
            }),
        })
    }

    pub(super) fn analyze_arrow_function<'arena, 'src>(
        &mut self,
        af: &ArrowFunctionExpr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        for param in af.params.iter() {
            if let Some(hint) = &param.type_hint {
                self.check_type_hint(hint);
            }
        }
        if let Some(hint) = &af.return_type {
            self.check_type_hint(hint);
        }

        let params = ast_params_to_fn_params_resolved(
            &af.params,
            ctx.self_fqcn.as_deref(),
            self.db,
            &self.file,
        );
        let return_ty_hint = af
            .return_type
            .as_ref()
            .map(|h| crate::parser::type_from_hint(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union(u, self.db, &self.file));

        let mut arrow_ctx = crate::context::Context::for_function(
            &params,
            return_ty_hint.clone(),
            ctx.self_fqcn.clone(),
            ctx.parent_fqcn.clone(),
            ctx.static_fqcn.clone(),
            ctx.strict_types,
            af.is_static,
        );
        for (name, ty) in &ctx.vars {
            if !arrow_ctx.vars.contains_key(name) {
                arrow_ctx.set_var(name, ty.clone());
            }
        }

        let inferred_return = self.analyze(af.body, &mut arrow_ctx);
        for name in &arrow_ctx.read_vars {
            ctx.read_vars.insert(name.clone());
        }

        let return_ty = return_ty_hint.unwrap_or(inferred_return);
        let closure_params: Vec<mir_types::atomic::FnParam> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: p.name.clone(),
                ty: p.ty.clone(),
                default: p.default.clone(),
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();

        Union::single(Atomic::TClosure {
            params: closure_params,
            return_type: Box::new(return_ty),
            this_type: if af.is_static {
                None
            } else {
                ctx.self_fqcn.clone().map(|f| {
                    Box::new(Union::single(Atomic::TNamedObject {
                        fqcn: f,
                        type_params: vec![],
                    }))
                })
            },
        })
    }
}
