use super::return_type::{
    declared_return_has_template, named_object_return_compatible, return_arrays_compatible,
};
/// Flow-control statement handlers extracted from `analyze_stmt`.
///
/// Each method corresponds to one match arm in the parent `analyze_stmt`.
use super::StatementsAnalyzer;

use mir_issues::{IssueKind, Location};
use mir_types::{Atomic, Union};

impl<'a> StatementsAnalyzer<'a> {
    // -----------------------------------------------------------------------
    // Return
    // -----------------------------------------------------------------------

    pub(super) fn analyze_return_stmt<'arena, 'src>(
        &mut self,
        opt_expr: &Option<&php_ast::ast::Expr<'arena, 'src>>,
        stmt_span: php_ast::Span,
        ctx: &mut crate::context::Context,
    ) {
        if let Some(expr) = opt_expr {
            let ret_ty = self.expr_analyzer(ctx).analyze(expr, ctx);

            // If there's a bare `@var Type` (no variable name) on the return statement,
            // use the annotated type for the return-type compatibility check.
            // `@var Type $name` with a variable name narrows the variable (handled in
            // analyze_stmts loop), not the return type.
            let check_ty = if let Some((None, var_ty)) = self.extract_var_annotation(stmt_span) {
                var_ty
            } else {
                ret_ty.clone()
            };

            // Check against declared return type
            if let Some(declared) = &ctx.fn_return_type.clone() {
                // Check return type compatibility. Special case: `void` functions must not
                // return any value (named_object_return_compatible considers TVoid compatible
                // with TNull, so handle void separately to avoid false suppression).
                if !declared.contains(|t| matches!(t, Atomic::TConditional { .. }))
                    && ((declared.is_void() && !check_ty.is_void() && !check_ty.is_mixed())
                        || (!check_ty.is_subtype_of_simple(declared)
                        && !declared.is_mixed()
                        && !check_ty.is_mixed()
                        && !named_object_return_compatible(&check_ty, declared, self.db, &self.file)
                        // Also check without null (handles `null|T` where T implements declared).
                        // Guard: if check_ty is purely null, remove_null() is empty and would
                        // vacuously return true, incorrectly suppressing the error.
                        && (check_ty.remove_null().is_empty() || !named_object_return_compatible(&check_ty.remove_null(), declared, self.db, &self.file))
                        && !declared_return_has_template(declared, self.db)
                        && !declared_return_has_template(&check_ty, self.db)
                        && !return_arrays_compatible(&check_ty, declared, self.db, &self.file)
                        // Skip coercions: declared is more specific than actual
                        && !declared.is_subtype_of_simple(&check_ty)
                        && !declared.remove_null().is_subtype_of_simple(&check_ty)
                        // Skip when actual is compatible after removing null/false.
                        // Guard against empty union (e.g. pure-null type): removing null
                        // from `null` alone gives an empty union which vacuously passes
                        // is_subtype_of_simple — that would incorrectly suppress the error.
                        && (check_ty.remove_null().is_empty() || !check_ty.remove_null().is_subtype_of_simple(declared))
                        && !check_ty.remove_false().is_subtype_of_simple(declared)
                        // Suppress LessSpecificReturnStatement (level 4): actual is a
                        // supertype of declared (not flagged at default error level).
                        && !named_object_return_compatible(declared, &check_ty, self.db, &self.file)
                        && !named_object_return_compatible(&declared.remove_null(), &check_ty.remove_null(), self.db, &self.file)))
                {
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
                }
            }
            self.return_types.push(ret_ty);
        } else {
            self.return_types.push(Union::single(Atomic::TVoid));
            // Bare `return;` from a non-void declared function is an error.
            if let Some(declared) = &ctx.fn_return_type.clone() {
                if !declared.is_void() && !declared.is_mixed() {
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
                            crate::parser::span_text(self.source, stmt_span).unwrap_or_default(),
                        ),
                    );
                }
            }
        }
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Throw
    // -----------------------------------------------------------------------

    pub(super) fn analyze_throw_stmt<'arena, 'src>(
        &mut self,
        expr: &php_ast::ast::Expr<'arena, 'src>,
        stmt_span: php_ast::Span,
        ctx: &mut crate::context::Context,
    ) {
        let thrown_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
        // Validate that the thrown type extends Throwable
        for atomic in &thrown_ty.types {
            match atomic {
                mir_types::Atomic::TNamedObject { fqcn, .. } => {
                    let resolved = crate::db::resolve_name_via_db(self.db, &self.file, fqcn);
                    let is_throwable = resolved == "Throwable"
                        || resolved == "Exception"
                        || resolved == "Error"
                        || fqcn.as_ref() == "Throwable"
                        || fqcn.as_ref() == "Exception"
                        || fqcn.as_ref() == "Error"
                        || crate::db::extends_or_implements_via_db(self.db, &resolved, "Throwable")
                        || crate::db::extends_or_implements_via_db(self.db, &resolved, "Exception")
                        || crate::db::extends_or_implements_via_db(self.db, &resolved, "Error")
                        || crate::db::extends_or_implements_via_db(self.db, fqcn, "Throwable")
                        || crate::db::extends_or_implements_via_db(self.db, fqcn, "Exception")
                        || crate::db::extends_or_implements_via_db(self.db, fqcn, "Error")
                        // Suppress if class has unknown ancestors (might be Throwable)
                        || crate::db::has_unknown_ancestor_via_db(self.db, &resolved)
                        || crate::db::has_unknown_ancestor_via_db(self.db, fqcn)
                        // Suppress if class is not in codebase at all (could be extension class)
                        || (!crate::db::type_exists_via_db(self.db, &resolved) && !crate::db::type_exists_via_db(self.db, fqcn));
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
                        let thrown_fqcn = if crate::db::type_exists_via_db(self.db, &resolved) {
                            &resolved
                        } else {
                            fqcn.as_ref()
                        };
                        if !crate::db::is_unchecked_exception_via_db(self.db, thrown_fqcn)
                            && !ctx.fn_declared_throws.iter().any(|declared| {
                                declared.as_ref() == thrown_fqcn
                                    || crate::db::extends_or_implements_via_db(
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
                    let resolved = crate::db::resolve_name_via_db(self.db, &self.file, fqcn);
                    let is_throwable = resolved == "Throwable"
                        || resolved == "Exception"
                        || resolved == "Error"
                        || crate::db::extends_or_implements_via_db(self.db, &resolved, "Throwable")
                        || crate::db::extends_or_implements_via_db(self.db, &resolved, "Exception")
                        || crate::db::extends_or_implements_via_db(self.db, &resolved, "Error")
                        || crate::db::extends_or_implements_via_db(self.db, fqcn, "Throwable")
                        || crate::db::extends_or_implements_via_db(self.db, fqcn, "Exception")
                        || crate::db::extends_or_implements_via_db(self.db, fqcn, "Error")
                        || crate::db::has_unknown_ancestor_via_db(self.db, &resolved)
                        || crate::db::has_unknown_ancestor_via_db(self.db, fqcn);
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
                        let thrown_fqcn = if crate::db::type_exists_via_db(self.db, &resolved) {
                            &resolved
                        } else {
                            fqcn.as_ref()
                        };
                        if !crate::db::is_unchecked_exception_via_db(self.db, thrown_fqcn)
                            && !ctx.fn_declared_throws.iter().any(|declared| {
                                declared.as_ref() == thrown_fqcn
                                    || crate::db::extends_or_implements_via_db(
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

    pub(super) fn analyze_break_stmt(&mut self, ctx: &mut crate::context::Context) {
        // Save the context at the break point so the post-loop context
        // accounts for this early-exit path.
        if let Some(break_ctxs) = self.break_ctx_stack.last_mut() {
            break_ctxs.push(ctx.clone());
        }
        // Context after an unconditional break is dead; don't continue
        // emitting issues for code after this point.
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Continue
    // -----------------------------------------------------------------------

    pub(super) fn analyze_continue_stmt(&mut self, ctx: &mut crate::context::Context) {
        // continue goes back to the loop condition — no context to save,
        // the widening pass already re-analyses the body.
        ctx.diverges = true;
    }

    // -----------------------------------------------------------------------
    // Unset
    // -----------------------------------------------------------------------

    pub(super) fn analyze_unset_stmt<'arena, 'src>(
        &mut self,
        vars: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Expr<'arena, 'src>>,
        ctx: &mut crate::context::Context,
    ) {
        for var in vars.iter() {
            if let php_ast::ast::ExprKind::Variable(name) = &var.kind {
                ctx.unset_var(name.as_str().trim_start_matches('$'));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Static variable declaration
    // -----------------------------------------------------------------------

    pub(super) fn analyze_static_var_stmt<'arena, 'src>(
        &mut self,
        vars: &php_ast::ast::ArenaVec<'arena, php_ast::ast::StaticVar<'arena, 'src>>,
        ctx: &mut crate::context::Context,
    ) {
        for sv in vars.iter() {
            let ty = Union::mixed(); // static vars are indeterminate on entry
            ctx.set_var(sv.name.to_string().trim_start_matches('$'), ty);
        }
    }

    // -----------------------------------------------------------------------
    // Global declaration
    // -----------------------------------------------------------------------

    pub(super) fn analyze_global_stmt<'arena, 'src>(
        &mut self,
        vars: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Expr<'arena, 'src>>,
        ctx: &mut crate::context::Context,
    ) {
        for var in vars.iter() {
            if let php_ast::ast::ExprKind::Variable(name) = &var.kind {
                let var_name = name.as_str().trim_start_matches('$');
                let ty = self
                    .db
                    .global_var_type(var_name)
                    .unwrap_or_else(Union::mixed);
                ctx.set_var(var_name, ty);
            }
        }
    }

    // -----------------------------------------------------------------------
    // Declare
    // -----------------------------------------------------------------------

    pub(super) fn analyze_declare_stmt<'arena, 'src>(
        &mut self,
        d: &php_ast::ast::DeclareStmt<'arena, 'src>,
        ctx: &mut crate::context::Context,
    ) {
        for (name, _val) in d.directives.iter() {
            if *name == "strict_types" {
                ctx.strict_types = true;
            }
        }
        if let Some(body) = &d.body {
            self.analyze_stmt(body, ctx);
        }
    }
}
