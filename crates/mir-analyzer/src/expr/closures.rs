use super::helpers::{
    apply_doc_param_types, ast_params_to_fn_params_resolved, resolve_named_objects_in_union,
};
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

/// Carry a `$this->prop` narrowing proven before a closure/arrow function
/// literal into the closure's own scope, but only for `readonly` properties.
/// An ordinary mutable property could still change between the guard and
/// whenever the closure actually runs, so resetting it is correct; a
/// `readonly` property can never change after construction, so the guard's
/// proof stays valid no matter when the closure is invoked.
fn propagate_readonly_prop_refinements(
    db: &dyn crate::db::MirDatabase,
    ctx: &FlowState,
    inner_ctx: &mut FlowState,
) {
    let Some(self_fqcn) = ctx.self_fqcn.clone() else {
        return;
    };
    let this_sym = mir_types::Name::from("this");
    let here = crate::db::Fqcn::from_str(db, self_fqcn.as_ref());
    for ((obj_var, prop), ty) in ctx.prop_refined.iter() {
        if *obj_var != this_sym {
            continue;
        }
        if let Some((_, p_def)) = crate::db::find_property_in_chain(db, here, prop.as_str()) {
            if p_def.is_readonly {
                inner_ctx.set_prop_refined("this", prop.as_str(), (**ty).clone());
            }
        }
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

        let leading_doc = crate::parser::find_preceding_docblock(self.source, expr_span.start)
            .map(|doc| crate::parser::DocblockParser::parse(&doc));

        let mut params = ast_params_to_fn_params_resolved(
            &c.params,
            ctx.self_fqcn.as_deref(),
            self.db,
            &self.file,
        );
        if let Some(doc) = &leading_doc {
            apply_doc_param_types(&mut params, &c.params, &doc.params, self.db, &self.file);
        }
        let return_ty_hint = c
            .return_type
            .as_ref()
            .map(|h| crate::parser::type_from_hint_owned(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union(u, self.db, &self.file))
            .or_else(|| {
                // Fall back to `@return` docblock preceding the `function` keyword.
                leading_doc
                    .as_ref()
                    .and_then(|doc| doc.return_type.clone())
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
        // Closures see the enclosing function/method's template params (e.g. a
        // captured `@template T`-typed variable assigned to a typed property
        // inside the closure body) — without this, `type_refs_any_template`
        // checks against an empty set and treats the value as a concrete type,
        // producing spurious InvalidPropertyAssignment/instanceof narrowing bugs.
        closure_ctx.template_param_names = Arc::clone(&ctx.template_param_names);
        // A closure invoked from inside a @pure/@psalm-immutable/
        // @psalm-external-mutation-free body can still smuggle out an
        // observable side effect, so it must inherit that purity context
        // rather than starting fresh — an immediately-invoked closure that
        // mutates a captured object would otherwise go completely unchecked.
        closure_ctx.is_in_pure_fn = ctx.is_in_pure_fn;
        closure_ctx.is_in_immutable_method = ctx.is_in_immutable_method;
        closure_ctx.is_in_external_mutation_free_method = ctx.is_in_external_mutation_free_method;
        propagate_readonly_prop_refinements(self.db, ctx, &mut closure_ctx);
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

        // A by-value capture of a variable that is itself a parameter of the
        // enclosing function is still externally owned by the caller, so
        // calling a mutating method on it inside the closure body is an
        // externally observable side effect — exactly like calling one on a
        // real parameter. Extend `param_names` so the existing pure/
        // immutable/external-mutation-free method-call checks (which key off
        // that set) also catch such captures. A capture of a locally-created
        // object stays out of this set, matching the "local objects are
        // exempt" rule the same checks already apply to real params.
        if closure_ctx.is_in_pure_fn
            || closure_ctx.is_in_immutable_method
            || closure_ctx.is_in_external_mutation_free_method
        {
            let mut extended_param_names = (*closure_ctx.param_names).clone();
            for use_var in c.use_vars.iter().filter(|uv| !uv.by_ref) {
                let name = use_var.name.trim_start_matches('$');
                if ctx.param_names.contains(&Name::from(name)) {
                    extended_param_names.insert(Name::from(name));
                }
            }
            closure_ctx.param_names = Arc::new(extended_param_names);
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

        sa.collect_symbols = self.collect_symbols;
        sa.analyze_stmts(&c.body.stmts, &mut closure_ctx);
        let inferred_return = crate::body_analysis::merge_return_types(&sa.return_types);
        // A closure containing `yield` always returns a Generator, regardless
        // of what (if anything) it `return`s — same inference as a top-level
        // function/method (see `build_generator_return_type`), which this
        // closure-local `sa` otherwise silently dropped by only reading
        // `return_types`.
        let inferred_return = if sa.yielded_types.is_empty() {
            inferred_return
        } else {
            crate::body_analysis::build_generator_return_type(&sa.yielded_types, inferred_return)
        };

        // If the closure reads an outer-scope variable without capturing it via `use`,
        // mark that variable as read in the outer context to suppress false UnusedParam.
        for name in &closure_ctx.read_vars {
            if ctx.var_is_defined(name) || ctx.var_possibly_defined(name) {
                ctx.read_vars.insert(*name);
                ctx.mark_consumed(name.as_str());
            }
        }

        let return_ty = return_ty_hint.unwrap_or(inferred_return);
        let closure_params: Box<[mir_types::atomic::FnParam]> = params
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
            data: Box::new(mir_types::atomic::ClosureData {
                params: closure_params,
                return_type: return_ty,
                this_type: ctx.self_fqcn.clone().map(|f| {
                    Type::single(Atomic::TNamedObject {
                        fqcn: Name::from(f.as_ref()),
                        type_params: mir_types::union::empty_type_params(),
                    })
                }),
            }),
        })
    }

    pub(super) fn analyze_arrow_function(
        &mut self,
        af: &ArrowFunctionExpr,
        expr_span: php_ast::Span,
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

        let leading_doc = crate::parser::find_preceding_docblock(self.source, expr_span.start)
            .map(|doc| crate::parser::DocblockParser::parse(&doc));

        let mut params = ast_params_to_fn_params_resolved(
            &af.params,
            ctx.self_fqcn.as_deref(),
            self.db,
            &self.file,
        );
        if let Some(doc) = &leading_doc {
            apply_doc_param_types(&mut params, &af.params, &doc.params, self.db, &self.file);
        }
        let return_ty_hint = af
            .return_type
            .as_ref()
            .map(|h| crate::parser::type_from_hint_owned(h, ctx.self_fqcn.as_deref()))
            .map(|u| resolve_named_objects_in_union(u, self.db, &self.file))
            .or_else(|| {
                // Fall back to `@return` docblock preceding the `fn` keyword — mirrors
                // the same fallback in `analyze_closure` for `function(...) {...}`.
                leading_doc
                    .as_ref()
                    .and_then(|doc| doc.return_type.clone())
                    .map(|ty| resolve_named_objects_in_union(ty, self.db, &self.file))
            });

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
        // See analyze_closure: propagate the enclosing scope's template params
        // so captured template-typed variables aren't misjudged as concrete.
        arrow_ctx.template_param_names = Arc::clone(&ctx.template_param_names);
        // See analyze_closure: an arrow function invoked from inside a
        // @pure/@psalm-immutable/@psalm-external-mutation-free body can still
        // smuggle out a side effect through an implicitly-captured variable —
        // `fn() => impure_fn()` or a tainted value flowing into a sink must be
        // checked the same way the equivalent `function(){...}` closure is.
        arrow_ctx.is_in_pure_fn = ctx.is_in_pure_fn;
        arrow_ctx.is_in_immutable_method = ctx.is_in_immutable_method;
        arrow_ctx.is_in_external_mutation_free_method = ctx.is_in_external_mutation_free_method;
        propagate_readonly_prop_refinements(self.db, ctx, &mut arrow_ctx);
        // Arrow functions auto-capture every outer variable by value (no
        // explicit `use()` list), so taint on any of them must carry over too.
        arrow_ctx.tainted_vars = ctx.tainted_vars.clone();
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
        // See analyze_closure: a captured (by-value) outer parameter is still
        // externally owned by the caller, so mutating it via method call
        // inside the arrow body is an observable side effect just like a real
        // parameter — extend param_names so the existing pure/immutable/
        // external-mutation-free checks (which key off that set) catch it.
        // Every outer var is auto-captured, so union the whole set rather
        // than filtering by an explicit use() list.
        if arrow_ctx.is_in_pure_fn
            || arrow_ctx.is_in_immutable_method
            || arrow_ctx.is_in_external_mutation_free_method
        {
            let mut extended_param_names = (*arrow_ctx.param_names).clone();
            extended_param_names.extend(ctx.param_names.iter().copied());
            arrow_ctx.param_names = Arc::new(extended_param_names);
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
        let closure_params: Box<[mir_types::atomic::FnParam]> = params
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
            data: Box::new(mir_types::atomic::ClosureData {
                params: closure_params,
                return_type: return_ty,
                this_type: if af.is_static {
                    None
                } else {
                    ctx.self_fqcn.clone().map(|f| {
                        Type::single(Atomic::TNamedObject {
                            fqcn: Name::from(f.as_ref()),
                            type_params: mir_types::union::empty_type_params(),
                        })
                    })
                },
            }),
        })
    }
}
