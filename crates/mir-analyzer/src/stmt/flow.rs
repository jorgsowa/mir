use super::return_type::{
    declared_return_has_template, named_object_return_compatible, return_arrays_compatible,
    scalar_return_coercion_ok,
};
/// Flow-control statement handlers extracted from `analyze_stmt`.
///
/// Each method corresponds to one match arm in the parent `analyze_stmt`.
use super::StatementsAnalyzer;

/// Returns true when `actual` does not satisfy `declared` and an InvalidReturnType
/// diagnostic should fire.  Combines scalar structural checks (fast path for primitives)
/// with codebase-aware named-object and array checks.
fn return_type_is_invalid(
    actual: &Type,
    declared: &Type,
    strict_types: bool,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> bool {
    // Fast path: scalar actual is already a structural subtype of declared.
    if actual.is_subtype_structural(declared) {
        return false;
    }
    if declared.is_mixed() || actual.is_mixed() {
        return false;
    }
    if named_object_return_compatible(actual, declared, db, file) {
        return false;
    }
    // Also check without null (handles `null|T` where T implements declared).
    // Guard: if actual is purely null, remove_null() is empty and would
    // vacuously return true, incorrectly suppressing the error.
    if !actual.remove_null().is_empty()
        && named_object_return_compatible(&actual.remove_null(), declared, db, file)
    {
        return false;
    }
    if declared_return_has_template(declared, db) || declared_return_has_template(actual, db) {
        return false;
    }
    if return_arrays_compatible(actual, declared, db, file) {
        return false;
    }
    // Non-strict scalar return coercion: PHP silently coerces int/false → bool
    // in non-strict files. Covers the idiomatic `return preg_match(...)` pattern.
    if !strict_types && scalar_return_coercion_ok(actual, declared) {
        return false;
    }
    // Scalar coercion suppression: declared is a structural subtype of actual
    // (declared is more specific — widening is not an error at this level).
    if declared.is_subtype_structural(actual)
        || declared.remove_null().is_subtype_structural(actual)
    {
        return false;
    }
    // Scalar strip suppression: actual without null/false is already compatible.
    // Guard against empty union (e.g. pure-null type): removing null from `null`
    // alone gives an empty union which vacuously passes — that would incorrectly
    // suppress the error.
    if !actual.remove_null().is_empty() && actual.remove_null().is_subtype_structural(declared) {
        return false;
    }
    if !actual.remove_false().is_empty() && actual.remove_false().is_subtype_structural(declared) {
        return false;
    }
    // Suppress LessSpecificReturnStatement (level 4): actual is a supertype of declared
    // (not flagged at default error level).
    if named_object_return_compatible(declared, actual, db, file) {
        return false;
    }
    if named_object_return_compatible(&declared.remove_null(), &actual.remove_null(), db, file) {
        return false;
    }
    true
}

use mir_issues::{IssueKind, Location};
use mir_types::{Atomic, Type};
use php_ast::owned::{Expr, StaticVar};

impl<'a> StatementsAnalyzer<'a> {
    // -----------------------------------------------------------------------
    // Return
    // -----------------------------------------------------------------------

    pub(super) fn analyze_return_stmt(
        &mut self,
        opt_expr: &Option<Box<Expr>>,
        stmt: &php_ast::owned::Stmt,
        ctx: &mut crate::flow_state::FlowState,
    ) {
        let stmt_span = stmt.span;
        if let Some(expr) = opt_expr {
            let ret_ty = self.expr_analyzer(ctx).analyze(expr, ctx);

            // If there's a bare `@var Type` (no variable name) on the return statement,
            // use the annotated type for the return-type compatibility check.
            // `@var Type $name` with a variable name narrows the variable (handled in
            // analyze_stmts loop), not the return type.
            let doc = crate::parser::find_preceding_docblock(self.source, stmt_span.start);
            let check_ty = if let Some(ann) = self.extract_var_annotation_from(doc.as_deref()) {
                if ann.name.is_none() {
                    ann.ty
                } else {
                    ret_ty.clone()
                }
            } else {
                ret_ty.clone()
            };

            // Check against declared return type
            if let Some(declared) = &ctx.fn_return_type.clone() {
                // Check return type compatibility. Special case: `void` functions must not
                // return any value (named_object_return_compatible considers TVoid compatible
                // with TNull, so handle void separately to avoid false suppression).
                let has_invalid = !declared.contains(|t| matches!(t, Atomic::TConditional { .. }))
                    && ((declared.is_void() && !check_ty.is_void() && !check_ty.is_mixed())
                        || return_type_is_invalid(
                            &check_ty,
                            declared,
                            ctx.strict_types,
                            self.db,
                            &self.file,
                        ));
                let is_mixed_return = !has_invalid
                    && !declared.is_void()
                    && !declared.is_mixed()
                    && check_ty.is_mixed()
                    && !declared.contains(|t| matches!(t, Atomic::TConditional { .. }));
                if is_mixed_return {
                    let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                    self.issues.add(
                        mir_issues::Issue::new(
                            IssueKind::MixedReturnStatement {
                                declared: format!("{declared}"),
                            },
                            Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        )
                        .with_snippet(
                            crate::parser::span_text(self.source, stmt_span).unwrap_or_default(),
                        ),
                    );
                } else if has_invalid {
                    let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                    self.issues.add(
                        mir_issues::Issue::new(
                            IssueKind::InvalidReturnType {
                                expected: format!("{declared}"),
                                actual: format!("{ret_ty}"),
                            },
                            Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        )
                        .with_snippet(
                            crate::parser::span_text(self.source, stmt_span).unwrap_or_default(),
                        ),
                    );
                } else if !declared.is_void()
                    && !declared.is_mixed()
                    && !declared.contains(|t| matches!(t, Atomic::TNull))
                    && !declared.contains(|t| matches!(t, Atomic::TConditional { .. }))
                    && check_ty.contains(|t| matches!(t, Atomic::TNull))
                    && !check_ty.remove_null().is_empty()
                    && !return_type_is_invalid(
                        &check_ty.remove_null(),
                        declared,
                        ctx.strict_types,
                        self.db,
                        &self.file,
                    )
                {
                    // The actual type contains null but declared doesn't allow it,
                    // and the non-null part is otherwise compatible → NullableReturnStatement.
                    let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                    self.issues.add(
                        mir_issues::Issue::new(
                            IssueKind::NullableReturnStatement {
                                expected: format!("{declared}"),
                                actual: format!("{ret_ty}"),
                            },
                            Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        )
                        .with_snippet(
                            crate::parser::span_text(self.source, stmt_span).unwrap_or_default(),
                        ),
                    );
                }
            }
            self.return_types.push(ret_ty);
        } else {
            self.return_types.push(Type::single(Atomic::TVoid));
            // Bare `return;` from a non-void declared function is an error,
            // except inside a generator where it terminates iteration. A bare
            // return yields null, so it is also valid when the declared type
            // allows null (`?T`) or includes `void` in a union (`T|void`) — in
            // both cases `is_void()` (single-atomic) misses it, so check
            // `contains` for `TVoid`/`TNull`.
            if !ctx.is_generator {
                if let Some(declared) = &ctx.fn_return_type.clone() {
                    if !declared.is_void()
                        && !declared.is_mixed()
                        && !declared.contains(|t| matches!(t, Atomic::TVoid | Atomic::TNull))
                    {
                        let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                        self.issues.add(
                            mir_issues::Issue::new(
                                IssueKind::InvalidReturnType {
                                    expected: format!("{declared}"),
                                    actual: "void".to_string(),
                                },
                                Location {
                                    file: self.file.clone(),
                                    line,
                                    line_end,
                                    col_start,
                                    col_end: col_end.max(col_start + 1),
                                },
                            )
                            .with_snippet(
                                crate::parser::span_text(self.source, stmt_span)
                                    .unwrap_or_default(),
                            ),
                        );
                    }
                }
            }
        }
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Throw
    // -----------------------------------------------------------------------

    pub(super) fn analyze_throw_stmt(
        &mut self,
        expr: &Expr,
        stmt_span: php_ast::Span,
        ctx: &mut crate::flow_state::FlowState,
    ) {
        let thrown_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
        // Validate that the thrown type extends Throwable
        for atomic in &thrown_ty.types {
            match atomic {
                mir_types::Atomic::TNamedObject { fqcn, .. } => {
                    let resolved = crate::db::resolve_name(self.db, &self.file, fqcn);
                    let is_throwable = resolved == "Throwable"
                        || resolved == "Exception"
                        || resolved == "Error"
                        || fqcn.as_ref() == "Throwable"
                        || fqcn.as_ref() == "Exception"
                        || fqcn.as_ref() == "Error"
                        || crate::db::extends_or_implements(self.db, &resolved, "Throwable")
                        || crate::db::extends_or_implements(self.db, &resolved, "Exception")
                        || crate::db::extends_or_implements(self.db, &resolved, "Error")
                        || crate::db::extends_or_implements(self.db, fqcn, "Throwable")
                        || crate::db::extends_or_implements(self.db, fqcn, "Exception")
                        || crate::db::extends_or_implements(self.db, fqcn, "Error")
                        // Suppress if class has unknown ancestors (might be Throwable)
                        || crate::db::has_unknown_ancestor(self.db, &resolved)
                        || crate::db::has_unknown_ancestor(self.db, fqcn)
                        // Suppress if class is not in codebase at all (could be extension class)
                        || (!crate::db::class_exists(self.db, &resolved) && !crate::db::class_exists(self.db, fqcn));
                    if !is_throwable {
                        let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                        self.issues.add(mir_issues::Issue::new(
                            IssueKind::InvalidThrow {
                                ty: fqcn.to_string(),
                            },
                            Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        ));
                    } else {
                        // Check if thrown exception is covered by @throws declarations
                        let thrown_fqcn = if crate::db::class_exists(self.db, &resolved) {
                            &resolved
                        } else {
                            fqcn.as_ref()
                        };
                        if !crate::db::is_unchecked_exception(self.db, thrown_fqcn)
                            && !ctx.fn_declared_throws.iter().any(|declared| {
                                declared.as_ref() == thrown_fqcn
                                    || crate::db::extends_or_implements(
                                        self.db,
                                        thrown_fqcn,
                                        declared.as_ref(),
                                    )
                            })
                        {
                            let (line, line_end, col_start, col_end) =
                                self.span_to_location(stmt_span);
                            self.issues.add(mir_issues::Issue::new(
                                IssueKind::MissingThrowsDocblock {
                                    class: thrown_fqcn.to_string(),
                                },
                                Location {
                                    file: self.file.clone(),
                                    line,
                                    line_end,
                                    col_start,
                                    col_end: col_end.max(col_start + 1),
                                },
                            ));
                        }
                    }
                }
                // self/static/parent resolve to the class itself — check via fqcn
                mir_types::Atomic::TSelf { fqcn }
                | mir_types::Atomic::TStaticObject { fqcn }
                | mir_types::Atomic::TParent { fqcn } => {
                    let resolved = crate::db::resolve_name(self.db, &self.file, fqcn);
                    let is_throwable = resolved == "Throwable"
                        || resolved == "Exception"
                        || resolved == "Error"
                        || crate::db::extends_or_implements(self.db, &resolved, "Throwable")
                        || crate::db::extends_or_implements(self.db, &resolved, "Exception")
                        || crate::db::extends_or_implements(self.db, &resolved, "Error")
                        || crate::db::extends_or_implements(self.db, fqcn, "Throwable")
                        || crate::db::extends_or_implements(self.db, fqcn, "Exception")
                        || crate::db::extends_or_implements(self.db, fqcn, "Error")
                        || crate::db::has_unknown_ancestor(self.db, &resolved)
                        || crate::db::has_unknown_ancestor(self.db, fqcn);
                    if !is_throwable {
                        let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                        self.issues.add(mir_issues::Issue::new(
                            IssueKind::InvalidThrow {
                                ty: fqcn.to_string(),
                            },
                            Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        ));
                    } else {
                        // Check if thrown exception is covered by @throws declarations
                        let thrown_fqcn = if crate::db::class_exists(self.db, &resolved) {
                            &resolved
                        } else {
                            fqcn.as_ref()
                        };
                        if !crate::db::is_unchecked_exception(self.db, thrown_fqcn)
                            && !ctx.fn_declared_throws.iter().any(|declared| {
                                declared.as_ref() == thrown_fqcn
                                    || crate::db::extends_or_implements(
                                        self.db,
                                        thrown_fqcn,
                                        declared.as_ref(),
                                    )
                            })
                        {
                            let (line, line_end, col_start, col_end) =
                                self.span_to_location(stmt_span);
                            self.issues.add(mir_issues::Issue::new(
                                IssueKind::MissingThrowsDocblock {
                                    class: thrown_fqcn.to_string(),
                                },
                                Location {
                                    file: self.file.clone(),
                                    line,
                                    line_end,
                                    col_start,
                                    col_end: col_end.max(col_start + 1),
                                },
                            ));
                        }
                    }
                }
                mir_types::Atomic::TMixed | mir_types::Atomic::TObject => {}
                _ => {
                    let (line, line_end, col_start, col_end) = self.span_to_location(stmt_span);
                    self.issues.add(mir_issues::Issue::new(
                        IssueKind::InvalidThrow {
                            ty: format!("{thrown_ty}"),
                        },
                        Location {
                            file: self.file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    ));
                }
            }
        }
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Break
    // -----------------------------------------------------------------------

    pub(super) fn analyze_break_stmt(&mut self, ctx: &mut crate::flow_state::FlowState) {
        // Save the context at the break point so the post-loop context
        // accounts for this early-exit path.
        if let Some(break_ctxs) = self.break_ctx_stack.last_mut() {
            break_ctxs.push(ctx.clone());
        }
        // FlowState after an unconditional break is dead; don't continue
        // emitting issues for code after this point.
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Continue
    // -----------------------------------------------------------------------

    pub(super) fn analyze_continue_stmt(&mut self, ctx: &mut crate::flow_state::FlowState) {
        // continue goes back to the loop condition — no context to save,
        // the widening pass already re-analyses the body.
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Unset
    // -----------------------------------------------------------------------

    pub(super) fn analyze_unset_stmt(
        &mut self,
        vars: &[php_ast::owned::Expr],
        ctx: &mut crate::flow_state::FlowState,
    ) {
        for var in vars.iter() {
            if let php_ast::owned::ExprKind::Variable(name) = &var.kind {
                ctx.unset_var(name.trim_start_matches('$'));
            } else {
                // `unset($arr[$key])` / `unset($obj->prop)`: analyze the target
                // so the variables it reads (e.g. the array-access key) count as
                // uses — otherwise a foreach value used only as an unset key is
                // wrongly reported UnusedForeachValue.
                self.expr_analyzer(ctx).analyze(var, ctx);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Static variable declaration
    // -----------------------------------------------------------------------

    pub(super) fn analyze_static_var_stmt(
        &mut self,
        vars: &[StaticVar],
        ctx: &mut crate::flow_state::FlowState,
    ) {
        for sv in vars.iter() {
            let ty = Type::mixed(); // static vars are indeterminate on entry
            let name_str = sv.name.as_deref().unwrap_or("").to_string();
            let name = name_str.trim_start_matches('$');
            // Purity check: using a static variable in a @pure function.
            if ctx.is_in_pure_fn {
                let (line, col_start) = self.offset_to_line_col(sv.span.start);
                let (line_end, col_end) = self.offset_to_line_col(sv.span.end);
                self.issues.add(mir_issues::Issue::new(
                    IssueKind::ImpureStaticVariable {
                        variable: name.to_string(),
                    },
                    Location {
                        file: self.file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: col_end.max(col_start + 1),
                    },
                ));
            }
            ctx.set_var(name, ty);
            let (line, col_start) = self.offset_to_line_col(sv.span.start);
            let (line_end, col_end) = self.offset_to_line_col(sv.span.end);
            ctx.record_var_location(name, line, col_start, line_end, col_end);
        }
    }

    // -----------------------------------------------------------------------
    // Global declaration
    // -----------------------------------------------------------------------

    pub(super) fn analyze_global_stmt(
        &mut self,
        vars: &[php_ast::owned::Expr],
        ctx: &mut crate::flow_state::FlowState,
    ) {
        for var in vars.iter() {
            if let php_ast::owned::ExprKind::Variable(name) = &var.kind {
                let var_name = name.trim_start_matches('$');
                // Purity check: using a global variable in a @pure function.
                if ctx.is_in_pure_fn {
                    let (line, col_start) = self.offset_to_line_col(var.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(var.span.end);
                    self.issues.add(mir_issues::Issue::new(
                        IssueKind::ImpureGlobalVariable {
                            variable: var_name.to_string(),
                        },
                        Location {
                            file: self.file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    ));
                }
                let ty = self
                    .db
                    .global_var_type(var_name)
                    .unwrap_or_else(Type::mixed);
                ctx.set_var(var_name, ty);
                std::sync::Arc::make_mut(&mut ctx.byref_param_names)
                    .insert(mir_types::Name::from(var_name));
                let (line, col_start) = self.offset_to_line_col(var.span.start);
                let (line_end, col_end) = self.offset_to_line_col(var.span.end);
                ctx.record_var_location(var_name, line, col_start, line_end, col_end);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Declare
    // -----------------------------------------------------------------------

    pub(super) fn analyze_declare_stmt(
        &mut self,
        d: &php_ast::owned::DeclareStmt,
        ctx: &mut crate::flow_state::FlowState,
    ) {
        for (name, _val) in d.directives.iter() {
            if name.as_ref() == "strict_types" {
                ctx.strict_types = true;
            }
        }
        if let Some(body) = &d.body {
            self.analyze_stmt(body, ctx);
        }
    }
}
