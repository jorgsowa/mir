use super::helpers::{ast_params_to_fn_params_resolved, resolve_named_objects_in_union};
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::stmt::{mir_check_matches, widen_for_check};
use crate::symbol::ReferenceKind;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Name, Type};
use php_ast::owned::{ArrowFunctionExpr, ClosureExpr, ExprKind, Param};
use php_ast::Span;
use std::sync::Arc;

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

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_closure(
        &mut self,
        c: &ClosureExpr,
        expr_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
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
            .map(|h| crate::parser::type_from_hint_owned(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union(u, self.db, &self.file))
            .or_else(|| {
                // Fall back to `@return` docblock preceding the `function` keyword.
                crate::parser::find_preceding_docblock(self.source, expr_span.start)
                    .and_then(|doc| crate::parser::DocblockParser::parse(&doc).return_type)
                    .map(|ty| resolve_named_objects_in_union(ty, self.db, &self.file))
            });

        if return_ty_hint.is_none() && self.mode == crate::expr::AnalysisMode::Full {
            self.emit(
                mir_issues::IssueKind::MissingClosureReturnType,
                mir_issues::Severity::Info,
                expr_span,
            );
        }

        let mut closure_ctx = crate::flow_state::FlowState::for_function(
            &params,
            return_ty_hint.clone(),
            Arc::from([]),
            ctx.self_fqcn.clone(),
            ctx.parent_fqcn.clone(),
            ctx.static_fqcn.clone(),
            ctx.strict_types,
            c.is_static,
        );
        for p in c.params.iter() {
            if let Some(raw) = p.name.as_deref() {
                let trimmed = raw.trim_start_matches('$');
                let ty = closure_ctx.get_var(trimmed);
                self.record_symbol(
                    param_name_span(self.source, p),
                    ReferenceKind::Variable(Arc::from(trimmed)),
                    ty,
                );
            }
        }
        for use_var in c.use_vars.iter() {
            let name = use_var.name.trim_start_matches('$');
            // A by-ref capture (`use (&$f)`) binds by reference and auto-creates
            // the variable in the parent scope if it does not yet exist, so it is
            // never "undefined" — this is what makes a self-referential closure
            // `$f = function () use (&$f) {...}` valid. Define it in both scopes
            // and skip the undefined check.
            if use_var.by_ref {
                if !ctx.var_is_defined(name) {
                    // Type an auto-created by-ref capture as a callable of
                    // unknown arity: the dominant case is the self-referential
                    // closure `$f = function () use (&$f)`, where `$f` is the
                    // closure being assigned. This avoids spurious
                    // MixedFunctionCall / arity errors when the body calls it.
                    ctx.set_var(
                        name,
                        Type::single(mir_types::Atomic::TCallable {
                            params: None,
                            return_type: None,
                        }),
                    );
                }
            } else if !ctx.var_is_defined(name) {
                if ctx.var_possibly_defined(name) {
                    self.emit(
                        mir_issues::IssueKind::PossiblyUndefinedVariable {
                            name: name.to_string(),
                        },
                        mir_issues::Severity::Warning,
                        use_var.span,
                    );
                } else {
                    self.emit(
                        mir_issues::IssueKind::UndefinedVariable {
                            name: name.to_string(),
                        },
                        mir_issues::Severity::Error,
                        use_var.span,
                    );
                }
            }
            closure_ctx.set_var(name, ctx.get_var(name));
            if ctx.is_tainted(name) {
                closure_ctx.taint_var(name);
            }
            // Mark the captured variable as read in the parent context, and
            // consume its pending write so it isn't reported as a dead write.
            ctx.read_vars.insert(mir_types::Name::from(name));
            ctx.mark_consumed(name);
        }

        let mut sa = crate::stmt::StatementsAnalyzer::new(
            self.db,
            self.file.clone(),
            self.source,
            self.source_map,
            self.issues,
            self.symbols,
            self.php_version,
            self.mode,
        );
        sa.analyze_stmts(&c.body.stmts, &mut closure_ctx);
        let inferred_return = crate::body_analysis::merge_return_types(&sa.return_types);

        // If the closure reads an outer-scope variable without capturing it via `use`,
        // mark that variable as read in the outer context to suppress false UnusedParam.
        for name in &closure_ctx.read_vars {
            if ctx.var_is_defined(name) || ctx.var_possibly_defined(name) {
                ctx.read_vars.insert(*name);
                ctx.mark_consumed(name.as_str());
            }
        }

        let return_ty = return_ty_hint.unwrap_or(inferred_return);
        let closure_params: Vec<mir_types::atomic::FnParam> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: Name::from(p.name.as_ref()),
                ty: p
                    .ty
                    .as_ref()
                    .map(|arc| mir_types::SimpleType::from_union((**arc).clone())),
                out_ty: None,
                default: if p.has_default {
                    Some(mir_types::SimpleType::from_union(Type::mixed()))
                } else {
                    None
                },
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();

        Type::single(Atomic::TClosure {
            params: closure_params,
            return_type: Box::new(return_ty),
            this_type: ctx.self_fqcn.clone().map(|f| {
                Box::new(Type::single(Atomic::TNamedObject {
                    fqcn: Name::from(f.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                }))
            }),
        })
    }

    pub(super) fn analyze_arrow_function(
        &mut self,
        af: &ArrowFunctionExpr,
        ctx: &mut FlowState,
    ) -> Type {
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
            .map(|h| crate::parser::type_from_hint_owned(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union(u, self.db, &self.file));

        let mut arrow_ctx = crate::flow_state::FlowState::for_function(
            &params,
            return_ty_hint.clone(),
            Arc::from([]),
            ctx.self_fqcn.clone(),
            ctx.parent_fqcn.clone(),
            ctx.static_fqcn.clone(),
            ctx.strict_types,
            af.is_static,
        );
        let this_sym = mir_types::Name::from("this");
        for (name, ty) in ctx.vars.iter() {
            // Static arrow functions don't capture $this from the outer scope.
            if af.is_static && *name == this_sym {
                continue;
            }
            if !arrow_ctx.vars.contains_key(name) {
                std::sync::Arc::make_mut(&mut arrow_ctx.vars).insert(*name, ty.clone());
                std::sync::Arc::make_mut(&mut arrow_ctx.assigned_vars).insert(*name);
            }
        }

        for p in af.params.iter() {
            if let Some(raw) = p.name.as_deref() {
                let trimmed = raw.trim_start_matches('$');
                // Use arrow_ctx.get_var to get the resolved type (params take priority
                // over outer-scope vars of the same name since they were inserted first).
                let ty = arrow_ctx.get_var(trimmed);
                self.record_symbol(
                    param_name_span(self.source, p),
                    ReferenceKind::Variable(Arc::from(trimmed)),
                    ty,
                );
            }
        }

        // Check @mir-check directives in the arrow function body.
        // If the body is parenthesized, look for docblocks before the inner expression.
        let check_target = match &af.body.kind {
            ExprKind::Parenthesized(inner) => inner.as_ref(),
            _ => &af.body,
        };
        if let Some(doc) =
            crate::parser::find_preceding_docblock(self.source, check_target.span.start)
        {
            let checks = crate::parser::DocblockParser::parse(&doc).mir_checks;
            for (var_name, expected_str) in checks {
                let expected = crate::parser::docblock::parse_type_string(&expected_str);
                let actual_raw = arrow_ctx.get_var(&var_name);
                if !mir_check_matches(&expected, &actual_raw) {
                    self.emit(
                        IssueKind::TypeCheckMismatch {
                            var: var_name,
                            expected: expected.to_string(),
                            actual: widen_for_check(actual_raw).to_string(),
                        },
                        Severity::Error,
                        check_target.span,
                    );
                }
            }
        }

        let inferred_return = self.analyze(&af.body, &mut arrow_ctx);
        // Arrow functions capture the whole outer scope by value: any variable
        // the body reads is a read (and consumed write) in the outer context.
        for name in &arrow_ctx.read_vars {
            ctx.read_vars.insert(*name);
            ctx.mark_consumed(name.as_str());
        }

        let return_ty = return_ty_hint.unwrap_or(inferred_return);
        let closure_params: Vec<mir_types::atomic::FnParam> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: Name::from(p.name.as_ref()),
                ty: p
                    .ty
                    .as_ref()
                    .map(|arc| mir_types::SimpleType::from_union((**arc).clone())),
                out_ty: None,
                default: if p.has_default {
                    Some(mir_types::SimpleType::from_union(Type::mixed()))
                } else {
                    None
                },
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();

        Type::single(Atomic::TClosure {
            params: closure_params,
            return_type: Box::new(return_ty),
            this_type: if af.is_static {
                None
            } else {
                ctx.self_fqcn.clone().map(|f| {
                    Box::new(Type::single(Atomic::TNamedObject {
                        fqcn: Name::from(f.as_ref()),
                        type_params: mir_types::union::empty_type_params(),
                    }))
                })
            },
        })
    }
}
