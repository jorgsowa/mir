/// Statement analyzer — walks statement nodes threading context through
/// control flow (if/else, loops, try/catch, return).
mod loops;
mod return_type;

use loops::{infer_foreach_types, vars_stabilized, widen_unstable};
pub(crate) use return_type::named_object_return_compatible;
use return_type::{declared_return_has_template, resolve_union_for_file, return_arrays_compatible};

use std::sync::Arc;

use php_ast::ast::StmtKind;

use mir_codebase::Codebase;
use mir_issues::{Issue, IssueBuffer, IssueKind, Location};
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::narrowing::narrow_from_condition;
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

// ---------------------------------------------------------------------------
// StatementsAnalyzer
// ---------------------------------------------------------------------------

pub struct StatementsAnalyzer<'a> {
    pub codebase: &'a Codebase,
    pub file: Arc<str>,
    pub source: &'a str,
    pub source_map: &'a php_rs_parser::source_map::SourceMap,
    pub issues: &'a mut IssueBuffer,
    pub symbols: &'a mut Vec<ResolvedSymbol>,
    pub php_version: PhpVersion,
    pub inference_only: bool,
    /// Accumulated inferred return types for the current function.
    pub return_types: Vec<Union>,
    /// Break-context stack: one entry per active loop nesting level.
    /// Each entry collects the context states at every `break` in that loop.
    break_ctx_stack: Vec<Vec<Context>>,
}

impl<'a> StatementsAnalyzer<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        codebase: &'a Codebase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
        issues: &'a mut IssueBuffer,
        symbols: &'a mut Vec<ResolvedSymbol>,
        php_version: PhpVersion,
        inference_only: bool,
    ) -> Self {
        Self {
            codebase,
            file,
            source,
            source_map,
            issues,
            symbols,
            php_version,
            inference_only,
            return_types: Vec::new(),
            break_ctx_stack: Vec::new(),
        }
    }

    pub fn analyze_stmts<'arena, 'src>(
        &mut self,
        stmts: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Stmt<'arena, 'src>>,
        ctx: &mut Context,
    ) {
        for stmt in stmts.iter() {
            // @psalm-suppress / @suppress per-statement (call-site suppression)
            let suppressions = self.extract_statement_suppressions(stmt.span);
            let before = self.issues.issue_count();

            if ctx.diverges {
                let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                    let (end_line, end_col) = self.offset_to_line_col(stmt.span.end);
                    (end_line, end_col)
                } else {
                    (line, col_start + 1)
                };
                self.issues.add(
                    Issue::new(
                        IssueKind::UnreachableCode,
                        Location {
                            file: self.file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    )
                    .with_snippet(
                        crate::parser::span_text(self.source, stmt.span).unwrap_or_default(),
                    ),
                );
                if !suppressions.is_empty() {
                    self.issues.suppress_range(before, &suppressions);
                }
                break;
            }

            // Extract @var annotation for this statement.
            let var_annotation = self.extract_var_annotation(stmt.span);

            // Pre-narrow: `@var Type $varname` before any statement narrows that variable.
            // Special cases: before `return` or before `foreach ... as $valvar` (value override).
            if let Some((Some(ref var_name), ref var_ty)) = var_annotation {
                ctx.set_var(var_name.as_str(), var_ty.clone());
            }

            self.analyze_stmt(stmt, ctx);

            // Post-narrow: `@var Type $varname` before `$varname = expr()` overrides
            // the inferred type with the annotated type. Only applies when the assignment
            // target IS the annotated variable.
            if let Some((Some(ref var_name), ref var_ty)) = var_annotation {
                if let php_ast::ast::StmtKind::Expression(e) = &stmt.kind {
                    if let php_ast::ast::ExprKind::Assign(a) = &e.kind {
                        if matches!(&a.op, php_ast::ast::AssignOp::Assign) {
                            if let php_ast::ast::ExprKind::Variable(lhs_name) = &a.target.kind {
                                let lhs = lhs_name.trim_start_matches('$');
                                if lhs == var_name.as_str() {
                                    ctx.set_var(var_name.as_str(), var_ty.clone());
                                }
                            }
                        }
                    }
                }
            }

            if !suppressions.is_empty() {
                self.issues.suppress_range(before, &suppressions);
            }
        }
    }

    pub fn analyze_stmt<'arena, 'src>(
        &mut self,
        stmt: &php_ast::ast::Stmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        match &stmt.kind {
            // ---- Expression statement ----------------------------------------
            StmtKind::Expression(expr) => {
                let expr_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
                if expr_ty.is_never() {
                    ctx.diverges = true;
                }
                // For standalone assert($condition) calls, narrow from the condition.
                if let php_ast::ast::ExprKind::FunctionCall(call) = &expr.kind {
                    if let php_ast::ast::ExprKind::Identifier(fn_name) = &call.name.kind {
                        if fn_name.eq_ignore_ascii_case("assert") {
                            if let Some(arg) = call.args.first() {
                                narrow_from_condition(
                                    &arg.value,
                                    ctx,
                                    true,
                                    self.codebase,
                                    &self.file,
                                );
                            }
                        }
                    }
                }
            }

            // ---- Echo ---------------------------------------------------------
            StmtKind::Echo(exprs) => {
                for expr in exprs.iter() {
                    // Taint check (M19): echoing tainted data → XSS
                    if crate::taint::is_expr_tainted(expr, ctx) {
                        let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                        let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                            let (end_line, end_col) = self.offset_to_line_col(stmt.span.end);
                            (end_line, end_col)
                        } else {
                            (line, col_start)
                        };
                        let mut issue = mir_issues::Issue::new(
                            IssueKind::TaintedHtml,
                            mir_issues::Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        );
                        // Extract snippet from the echo statement span.
                        let start = stmt.span.start as usize;
                        let end = stmt.span.end as usize;
                        if start < self.source.len() {
                            let end = end.min(self.source.len());
                            let span_text = &self.source[start..end];
                            if let Some(first_line) = span_text.lines().next() {
                                issue = issue.with_snippet(first_line.trim().to_string());
                            }
                        }
                        self.issues.add(issue);
                    }
                    self.expr_analyzer(ctx).analyze(expr, ctx);
                }
            }

            // ---- Return -------------------------------------------------------
            StmtKind::Return(opt_expr) => {
                if let Some(expr) = opt_expr {
                    let ret_ty = self.expr_analyzer(ctx).analyze(expr, ctx);

                    // If there's a bare `@var Type` (no variable name) on the return statement,
                    // use the annotated type for the return-type compatibility check.
                    // `@var Type $name` with a variable name narrows the variable (handled in
                    // analyze_stmts loop), not the return type.
                    let check_ty =
                        if let Some((None, var_ty)) = self.extract_var_annotation(stmt.span) {
                            var_ty
                        } else {
                            ret_ty.clone()
                        };

                    // Check against declared return type
                    if let Some(declared) = &ctx.fn_return_type.clone() {
                        // Check return type compatibility. Special case: `void` functions must not
                        // return any value (named_object_return_compatible considers TVoid compatible
                        // with TNull, so handle void separately to avoid false suppression).
                        if (declared.is_void() && !check_ty.is_void() && !check_ty.is_mixed())
                            || (!check_ty.is_subtype_of_simple(declared)
                                && !declared.is_mixed()
                                && !check_ty.is_mixed()
                                && !named_object_return_compatible(&check_ty, declared, self.codebase, &self.file)
                                // Also check without null (handles `null|T` where T implements declared).
                                // Guard: if check_ty is purely null, remove_null() is empty and would
                                // vacuously return true, incorrectly suppressing the error.
                                && (check_ty.remove_null().is_empty() || !named_object_return_compatible(&check_ty.remove_null(), declared, self.codebase, &self.file))
                                && !declared_return_has_template(declared, self.codebase)
                                && !declared_return_has_template(&check_ty, self.codebase)
                                && !return_arrays_compatible(&check_ty, declared, self.codebase, &self.file)
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
                                && !named_object_return_compatible(declared, &check_ty, self.codebase, &self.file)
                                && !named_object_return_compatible(&declared.remove_null(), &check_ty.remove_null(), self.codebase, &self.file))
                        {
                            let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                            let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                                let (end_line, end_col) = self.offset_to_line_col(stmt.span.end);
                                (end_line, end_col)
                            } else {
                                (line, col_start)
                            };
                            self.issues.add(
                                mir_issues::Issue::new(
                                    IssueKind::InvalidReturnType {
                                        expected: format!("{declared}"),
                                        actual: format!("{ret_ty}"),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        line_end,
                                        col_start,
                                        col_end: col_end.max(col_start + 1),
                                    },
                                )
                                .with_snippet(
                                    crate::parser::span_text(self.source, stmt.span)
                                        .unwrap_or_default(),
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
                            let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                            let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                                let (end_line, end_col) = self.offset_to_line_col(stmt.span.end);
                                (end_line, end_col)
                            } else {
                                (line, col_start)
                            };
                            self.issues.add(
                                mir_issues::Issue::new(
                                    IssueKind::InvalidReturnType {
                                        expected: format!("{declared}"),
                                        actual: "void".to_string(),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        line_end,
                                        col_start,
                                        col_end: col_end.max(col_start + 1),
                                    },
                                )
                                .with_snippet(
                                    crate::parser::span_text(self.source, stmt.span)
                                        .unwrap_or_default(),
                                ),
                            );
                        }
                    }
                }
                ctx.diverges = true;
            }

            // ---- Throw --------------------------------------------------------
            StmtKind::Throw(expr) => {
                let thrown_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
                // Validate that the thrown type extends Throwable
                for atomic in &thrown_ty.types {
                    match atomic {
                        mir_types::Atomic::TNamedObject { fqcn, .. } => {
                            let resolved = self.codebase.resolve_class_name(&self.file, fqcn);
                            let is_throwable = resolved == "Throwable"
                                || resolved == "Exception"
                                || resolved == "Error"
                                || fqcn.as_ref() == "Throwable"
                                || fqcn.as_ref() == "Exception"
                                || fqcn.as_ref() == "Error"
                                || self.codebase.extends_or_implements(&resolved, "Throwable")
                                || self.codebase.extends_or_implements(&resolved, "Exception")
                                || self.codebase.extends_or_implements(&resolved, "Error")
                                || self.codebase.extends_or_implements(fqcn, "Throwable")
                                || self.codebase.extends_or_implements(fqcn, "Exception")
                                || self.codebase.extends_or_implements(fqcn, "Error")
                                // Suppress if class has unknown ancestors (might be Throwable)
                                || self.codebase.has_unknown_ancestor(&resolved)
                                || self.codebase.has_unknown_ancestor(fqcn)
                                // Suppress if class is not in codebase at all (could be extension class)
                                || (!self.codebase.type_exists(&resolved) && !self.codebase.type_exists(fqcn));
                            if !is_throwable {
                                let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                                let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                                    let (end_line, end_col) =
                                        self.offset_to_line_col(stmt.span.end);
                                    (end_line, end_col)
                                } else {
                                    (line, col_start)
                                };
                                self.issues.add(mir_issues::Issue::new(
                                    IssueKind::InvalidThrow {
                                        ty: fqcn.to_string(),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        line_end,
                                        col_start,
                                        col_end: col_end.max(col_start + 1),
                                    },
                                ));
                            }
                        }
                        // self/static/parent resolve to the class itself — check via fqcn
                        mir_types::Atomic::TSelf { fqcn }
                        | mir_types::Atomic::TStaticObject { fqcn }
                        | mir_types::Atomic::TParent { fqcn } => {
                            let resolved = self.codebase.resolve_class_name(&self.file, fqcn);
                            let is_throwable = resolved == "Throwable"
                                || resolved == "Exception"
                                || resolved == "Error"
                                || self.codebase.extends_or_implements(&resolved, "Throwable")
                                || self.codebase.extends_or_implements(&resolved, "Exception")
                                || self.codebase.extends_or_implements(&resolved, "Error")
                                || self.codebase.extends_or_implements(fqcn, "Throwable")
                                || self.codebase.extends_or_implements(fqcn, "Exception")
                                || self.codebase.extends_or_implements(fqcn, "Error")
                                || self.codebase.has_unknown_ancestor(&resolved)
                                || self.codebase.has_unknown_ancestor(fqcn);
                            if !is_throwable {
                                let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                                let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                                    let (end_line, end_col) =
                                        self.offset_to_line_col(stmt.span.end);
                                    (end_line, end_col)
                                } else {
                                    (line, col_start)
                                };
                                self.issues.add(mir_issues::Issue::new(
                                    IssueKind::InvalidThrow {
                                        ty: fqcn.to_string(),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        line_end,
                                        col_start,
                                        col_end: col_end.max(col_start + 1),
                                    },
                                ));
                            }
                        }
                        mir_types::Atomic::TMixed | mir_types::Atomic::TObject => {}
                        _ => {
                            let (line, col_start) = self.offset_to_line_col(stmt.span.start);
                            let (line_end, col_end) = if stmt.span.start < stmt.span.end {
                                let (end_line, end_col) = self.offset_to_line_col(stmt.span.end);
                                (end_line, end_col)
                            } else {
                                (line, col_start)
                            };
                            self.issues.add(mir_issues::Issue::new(
                                IssueKind::InvalidThrow {
                                    ty: format!("{thrown_ty}"),
                                },
                                mir_issues::Location {
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

            // ---- If -----------------------------------------------------------
            StmtKind::If(if_stmt) => {
                let pre_ctx = ctx.clone();

                // Analyse condition expression
                let cond_type = self.expr_analyzer(ctx).analyze(&if_stmt.condition, ctx);
                let pre_diverges = ctx.diverges;

                // True branch
                let mut then_ctx = ctx.fork();
                narrow_from_condition(
                    &if_stmt.condition,
                    &mut then_ctx,
                    true,
                    self.codebase,
                    &self.file,
                );
                // Capture narrowing-only unreachability before body analysis —
                // body divergence (continue/return/throw) must not trigger
                // RedundantCondition for valid conditions.
                let then_unreachable_from_narrowing = then_ctx.diverges;
                // Skip analyzing a statically-unreachable branch (prevents false
                // positives in dead branches caused by overly conservative types).
                if !then_ctx.diverges {
                    self.analyze_stmt(if_stmt.then_branch, &mut then_ctx);
                }

                // ElseIf branches (flatten into separate else-if chain)
                let mut elseif_ctxs: Vec<Context> = vec![];
                for elseif in if_stmt.elseif_branches.iter() {
                    // Start from the pre-if context narrowed by the if condition being false
                    // (an elseif body only runs when the if condition is false).
                    let mut pre_elseif = ctx.fork();
                    narrow_from_condition(
                        &if_stmt.condition,
                        &mut pre_elseif,
                        false,
                        self.codebase,
                        &self.file,
                    );
                    let pre_elseif_diverges = pre_elseif.diverges;

                    // Check reachability of the elseif body (condition narrowed true)
                    // and its implicit "skip" path (condition narrowed false) to detect
                    // redundant elseif conditions.
                    let mut elseif_true_ctx = pre_elseif.clone();
                    narrow_from_condition(
                        &elseif.condition,
                        &mut elseif_true_ctx,
                        true,
                        self.codebase,
                        &self.file,
                    );
                    let mut elseif_false_ctx = pre_elseif.clone();
                    narrow_from_condition(
                        &elseif.condition,
                        &mut elseif_false_ctx,
                        false,
                        self.codebase,
                        &self.file,
                    );
                    if !pre_elseif_diverges
                        && (elseif_true_ctx.diverges || elseif_false_ctx.diverges)
                    {
                        let (line, col_start) =
                            self.offset_to_line_col(elseif.condition.span.start);
                        let (line_end, col_end) =
                            if elseif.condition.span.start < elseif.condition.span.end {
                                let (end_line, end_col) =
                                    self.offset_to_line_col(elseif.condition.span.end);
                                (end_line, end_col)
                            } else {
                                (line, col_start)
                            };
                        let elseif_cond_type = self
                            .expr_analyzer(ctx)
                            .analyze(&elseif.condition, &mut ctx.fork());
                        self.issues.add(
                            mir_issues::Issue::new(
                                IssueKind::RedundantCondition {
                                    ty: format!("{elseif_cond_type}"),
                                },
                                mir_issues::Location {
                                    file: self.file.clone(),
                                    line,
                                    line_end,
                                    col_start,
                                    col_end: col_end.max(col_start + 1),
                                },
                            )
                            .with_snippet(
                                crate::parser::span_text(self.source, elseif.condition.span)
                                    .unwrap_or_default(),
                            ),
                        );
                    }

                    // Analyze the elseif body using the narrowed-true context.
                    let mut branch_ctx = elseif_true_ctx;
                    self.expr_analyzer(&branch_ctx)
                        .analyze(&elseif.condition, &mut branch_ctx);
                    if !branch_ctx.diverges {
                        self.analyze_stmt(&elseif.body, &mut branch_ctx);
                    }
                    elseif_ctxs.push(branch_ctx);
                }

                // Else branch
                let mut else_ctx = ctx.fork();
                narrow_from_condition(
                    &if_stmt.condition,
                    &mut else_ctx,
                    false,
                    self.codebase,
                    &self.file,
                );
                let else_unreachable_from_narrowing = else_ctx.diverges;
                if !else_ctx.diverges {
                    if let Some(else_branch) = &if_stmt.else_branch {
                        self.analyze_stmt(else_branch, &mut else_ctx);
                    }
                }

                // Emit RedundantCondition if narrowing proves one branch is statically unreachable.
                if !pre_diverges
                    && (then_unreachable_from_narrowing || else_unreachable_from_narrowing)
                {
                    let (line, col_start) = self.offset_to_line_col(if_stmt.condition.span.start);
                    let (line_end, col_end) =
                        if if_stmt.condition.span.start < if_stmt.condition.span.end {
                            let (end_line, end_col) =
                                self.offset_to_line_col(if_stmt.condition.span.end);
                            (end_line, end_col)
                        } else {
                            (line, col_start)
                        };
                    self.issues.add(
                        mir_issues::Issue::new(
                            IssueKind::RedundantCondition {
                                ty: format!("{cond_type}"),
                            },
                            mir_issues::Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: col_end.max(col_start + 1),
                            },
                        )
                        .with_snippet(
                            crate::parser::span_text(self.source, if_stmt.condition.span)
                                .unwrap_or_default(),
                        ),
                    );
                }

                // Merge all branches: start with the if/else pair, then fold each
                // elseif in as an additional possible execution path.  Using the
                // accumulated ctx (not pre_ctx) as the "else" argument ensures every
                // branch contributes to the final type environment.
                *ctx = Context::merge_branches(&pre_ctx, then_ctx, Some(else_ctx));
                for ec in elseif_ctxs {
                    *ctx = Context::merge_branches(&pre_ctx, ec, Some(ctx.clone()));
                }
            }

            // ---- While --------------------------------------------------------
            StmtKind::While(w) => {
                self.expr_analyzer(ctx).analyze(&w.condition, ctx);
                let pre = ctx.clone();

                // Entry context: narrow on true condition
                let mut entry = ctx.fork();
                narrow_from_condition(&w.condition, &mut entry, true, self.codebase, &self.file);

                let post = self.analyze_loop_widened(&pre, entry, |sa, iter| {
                    sa.analyze_stmt(w.body, iter);
                    sa.expr_analyzer(iter).analyze(&w.condition, iter);
                });
                *ctx = post;
            }

            // ---- Do-while -----------------------------------------------------
            StmtKind::DoWhile(dw) => {
                let pre = ctx.clone();
                let entry = ctx.fork();
                let post = self.analyze_loop_widened(&pre, entry, |sa, iter| {
                    sa.analyze_stmt(dw.body, iter);
                    sa.expr_analyzer(iter).analyze(&dw.condition, iter);
                });
                *ctx = post;
            }

            // ---- For ----------------------------------------------------------
            StmtKind::For(f) => {
                // Init expressions run once before the loop
                for init in f.init.iter() {
                    self.expr_analyzer(ctx).analyze(init, ctx);
                }
                let pre = ctx.clone();
                let mut entry = ctx.fork();
                for cond in f.condition.iter() {
                    self.expr_analyzer(&entry).analyze(cond, &mut entry);
                }

                let post = self.analyze_loop_widened(&pre, entry, |sa, iter| {
                    sa.analyze_stmt(f.body, iter);
                    for update in f.update.iter() {
                        sa.expr_analyzer(iter).analyze(update, iter);
                    }
                    for cond in f.condition.iter() {
                        sa.expr_analyzer(iter).analyze(cond, iter);
                    }
                });
                *ctx = post;
            }

            // ---- Foreach ------------------------------------------------------
            StmtKind::Foreach(fe) => {
                let arr_ty = self.expr_analyzer(ctx).analyze(&fe.expr, ctx);
                let (key_ty, mut value_ty) = infer_foreach_types(&arr_ty);

                // Apply `@var Type $varname` annotation on the foreach value variable.
                // The annotation always wins — it is the developer's explicit type assertion.
                if let Some(vname) = crate::expr::extract_simple_var(&fe.value) {
                    if let Some((Some(ann_var), ann_ty)) = self.extract_var_annotation(stmt.span) {
                        if ann_var == vname {
                            value_ty = ann_ty;
                        }
                    }
                }

                let pre = ctx.clone();
                let mut entry = ctx.fork();

                // Bind key variable on loop entry
                if let Some(key_expr) = &fe.key {
                    if let Some(var_name) = crate::expr::extract_simple_var(key_expr) {
                        entry.set_var(var_name, key_ty.clone());
                    }
                }
                // Bind value variable on loop entry.
                // The value may be a simple variable or a list/array destructure pattern.
                let value_var = crate::expr::extract_simple_var(&fe.value);
                let value_destructure_vars = crate::expr::extract_destructure_vars(&fe.value);
                if let Some(ref vname) = value_var {
                    entry.set_var(vname.as_str(), value_ty.clone());
                } else {
                    for vname in &value_destructure_vars {
                        entry.set_var(vname, Union::mixed());
                    }
                }

                let post = self.analyze_loop_widened(&pre, entry, |sa, iter| {
                    // Re-bind key/value each iteration (array may change)
                    if let Some(key_expr) = &fe.key {
                        if let Some(var_name) = crate::expr::extract_simple_var(key_expr) {
                            iter.set_var(var_name, key_ty.clone());
                        }
                    }
                    if let Some(ref vname) = value_var {
                        iter.set_var(vname.as_str(), value_ty.clone());
                    } else {
                        for vname in &value_destructure_vars {
                            iter.set_var(vname, Union::mixed());
                        }
                    }
                    sa.analyze_stmt(fe.body, iter);
                });
                *ctx = post;
            }

            // ---- Switch -------------------------------------------------------
            StmtKind::Switch(sw) => {
                let _subject_ty = self.expr_analyzer(ctx).analyze(&sw.expr, ctx);
                // Extract the subject variable name for narrowing (if it's a simple var)
                let subject_var: Option<String> = match &sw.expr.kind {
                    php_ast::ast::ExprKind::Variable(name) => {
                        Some(name.as_str().trim_start_matches('$').to_string())
                    }
                    _ => None,
                };
                // Detect `switch(true)` — case conditions are used as narrowing expressions
                let switch_on_true = matches!(&sw.expr.kind, php_ast::ast::ExprKind::Bool(true));

                let pre_ctx = ctx.clone();
                // Push a break-context bucket so that `break` inside cases saves
                // the case's context for merging into the post-switch result.
                self.break_ctx_stack.push(Vec::new());

                let has_default = sw.cases.iter().any(|c| c.value.is_none());

                // First pass: analyse each case body independently from pre_ctx.
                // Break statements inside a body save their context to break_ctx_stack
                // automatically; we just collect the per-case contexts here.
                let mut case_results: Vec<Context> = Vec::new();
                for case in sw.cases.iter() {
                    let mut case_ctx = pre_ctx.fork();
                    if let Some(val) = &case.value {
                        if switch_on_true {
                            // `switch(true) { case $x instanceof Y: }` — narrow from condition
                            narrow_from_condition(
                                val,
                                &mut case_ctx,
                                true,
                                self.codebase,
                                &self.file,
                            );
                        } else if let Some(ref var_name) = subject_var {
                            // Narrow subject var to the literal type of the case value
                            let narrow_ty = match &val.kind {
                                php_ast::ast::ExprKind::Int(n) => {
                                    Some(Union::single(Atomic::TLiteralInt(*n)))
                                }
                                php_ast::ast::ExprKind::String(s) => {
                                    Some(Union::single(Atomic::TLiteralString(Arc::from(&**s))))
                                }
                                php_ast::ast::ExprKind::Bool(b) => Some(Union::single(if *b {
                                    Atomic::TTrue
                                } else {
                                    Atomic::TFalse
                                })),
                                php_ast::ast::ExprKind::Null => Some(Union::single(Atomic::TNull)),
                                _ => None,
                            };
                            if let Some(narrowed) = narrow_ty {
                                case_ctx.set_var(var_name, narrowed);
                            }
                        }
                        self.expr_analyzer(&case_ctx).analyze(val, &mut case_ctx);
                    }
                    self.analyze_stmts(&case.body, &mut case_ctx);
                    case_results.push(case_ctx);
                }

                // Second pass: propagate divergence backwards through the fallthrough
                // chain. A non-diverging case (no break/return/throw) flows into the
                // next case at runtime, so if that next case effectively diverges, this
                // case effectively diverges too.
                //
                // Example:
                //   case 1: $y = "a";   // no break — chains into case 2
                //   case 2: return;     // diverges
                //
                // Case 1 is effectively diverging because its only exit is through
                // case 2's return. Adding case 1 to fallthrough_ctxs would be wrong.
                let n = case_results.len();
                let mut effective_diverges = vec![false; n];
                for i in (0..n).rev() {
                    if case_results[i].diverges {
                        effective_diverges[i] = true;
                    } else if i + 1 < n {
                        // Non-diverging body: falls through to the next case.
                        effective_diverges[i] = effective_diverges[i + 1];
                    }
                    // else: last case with no break/return — falls to end of switch.
                }

                // Build fallthrough_ctxs from cases that truly exit via the end of
                // the switch (not through a subsequent diverging case).
                let mut all_cases_diverge = true;
                let mut fallthrough_ctxs: Vec<Context> = Vec::new();
                for (i, case_ctx) in case_results.into_iter().enumerate() {
                    if !effective_diverges[i] {
                        all_cases_diverge = false;
                        fallthrough_ctxs.push(case_ctx);
                    }
                }

                // Pop break contexts — each `break` in a case body pushed its
                // context here, representing that case's effect on post-switch state.
                let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();

                // Build the post-switch merged context:
                // Start with pre_ctx if no default case (switch might not match anything)
                // or if not all cases diverge via return/throw.
                let mut merged = if has_default
                    && all_cases_diverge
                    && break_ctxs.is_empty()
                    && fallthrough_ctxs.is_empty()
                {
                    // All paths return/throw — post-switch is unreachable
                    let mut m = pre_ctx.clone();
                    m.diverges = true;
                    m
                } else {
                    // Start from pre_ctx (covers the "no case matched" path when there
                    // is no default, plus ensures pre-existing variables are preserved).
                    pre_ctx.clone()
                };

                for bctx in break_ctxs {
                    merged = Context::merge_branches(&pre_ctx, bctx, Some(merged));
                }
                for fctx in fallthrough_ctxs {
                    merged = Context::merge_branches(&pre_ctx, fctx, Some(merged));
                }

                *ctx = merged;
            }

            // ---- Try/catch/finally -------------------------------------------
            StmtKind::TryCatch(tc) => {
                let pre_ctx = ctx.clone();
                let mut try_ctx = ctx.fork();
                self.analyze_stmts(&tc.body, &mut try_ctx);

                // Build a base context for catch blocks that merges pre and try contexts.
                // Variables that might have been set during the try body are "possibly assigned"
                // in the catch (they may or may not have been set before the exception fired).
                let catch_base = Context::merge_branches(&pre_ctx, try_ctx.clone(), None);

                let mut non_diverging_catches: Vec<Context> = vec![];
                for catch in tc.catches.iter() {
                    let mut catch_ctx = catch_base.clone();
                    // Check that all caught exception types exist.
                    for catch_ty in catch.types.iter() {
                        self.check_name_undefined_class(catch_ty);
                    }
                    if let Some(var) = catch.var {
                        // Bind the caught exception variable; union all caught types
                        let exc_ty = if catch.types.is_empty() {
                            Union::single(Atomic::TObject)
                        } else {
                            let mut u = Union::empty();
                            for catch_ty in catch.types.iter() {
                                let raw = crate::parser::name_to_string(catch_ty);
                                let resolved = self.codebase.resolve_class_name(&self.file, &raw);
                                u.add_type(Atomic::TNamedObject {
                                    fqcn: resolved.into(),
                                    type_params: vec![],
                                });
                            }
                            u
                        };
                        catch_ctx.set_var(var.trim_start_matches('$'), exc_ty);
                    }
                    self.analyze_stmts(&catch.body, &mut catch_ctx);
                    if !catch_ctx.diverges {
                        non_diverging_catches.push(catch_ctx);
                    }
                }

                // If ALL catch branches diverge (return/throw/continue/break),
                // code after the try/catch is only reachable from the try body.
                // Use try_ctx directly so variables assigned in try are definitely set.
                let mut result = if non_diverging_catches.is_empty() {
                    let mut r = try_ctx;
                    r.diverges = false; // the try body itself may not have diverged
                    r
                } else {
                    // Some catches don't diverge — merge try with all non-diverging catches.
                    // Chain the merges: start with try_ctx, then fold in each catch branch.
                    let mut r = try_ctx;
                    for catch_ctx in non_diverging_catches {
                        r = Context::merge_branches(&pre_ctx, r, Some(catch_ctx));
                    }
                    r
                };

                // Finally runs unconditionally — analyze but don't merge vars
                if let Some(finally_stmts) = &tc.finally {
                    let mut finally_ctx = result.clone();
                    finally_ctx.inside_finally = true;
                    self.analyze_stmts(finally_stmts, &mut finally_ctx);
                    if finally_ctx.diverges {
                        result.diverges = true;
                    }
                }

                *ctx = result;
            }

            // ---- Block --------------------------------------------------------
            StmtKind::Block(stmts) => {
                self.analyze_stmts(stmts, ctx);
            }

            // ---- Break --------------------------------------------------------
            StmtKind::Break(_) => {
                // Save the context at the break point so the post-loop context
                // accounts for this early-exit path.
                if let Some(break_ctxs) = self.break_ctx_stack.last_mut() {
                    break_ctxs.push(ctx.clone());
                }
                // Context after an unconditional break is dead; don't continue
                // emitting issues for code after this point.
                ctx.diverges = true;
            }

            // ---- Continue ----------------------------------------------------
            StmtKind::Continue(_) => {
                // continue goes back to the loop condition — no context to save,
                // the widening pass already re-analyses the body.
                ctx.diverges = true;
            }

            // ---- Unset --------------------------------------------------------
            StmtKind::Unset(vars) => {
                for var in vars.iter() {
                    if let php_ast::ast::ExprKind::Variable(name) = &var.kind {
                        ctx.unset_var(name.as_str().trim_start_matches('$'));
                    }
                }
            }

            // ---- Static variable declaration ---------------------------------
            StmtKind::StaticVar(vars) => {
                for sv in vars.iter() {
                    let ty = Union::mixed(); // static vars are indeterminate on entry
                    ctx.set_var(sv.name.trim_start_matches('$'), ty);
                }
            }

            // ---- Global declaration ------------------------------------------
            StmtKind::Global(vars) => {
                for var in vars.iter() {
                    if let php_ast::ast::ExprKind::Variable(name) = &var.kind {
                        let var_name = name.as_str().trim_start_matches('$');
                        let ty = self
                            .codebase
                            .global_vars
                            .get(var_name)
                            .map(|r| r.clone())
                            .unwrap_or_else(Union::mixed);
                        ctx.set_var(var_name, ty);
                    }
                }
            }

            // ---- Declare -----------------------------------------------------
            StmtKind::Declare(d) => {
                for (name, _val) in d.directives.iter() {
                    if *name == "strict_types" {
                        ctx.strict_types = true;
                    }
                }
                if let Some(body) = &d.body {
                    self.analyze_stmt(body, ctx);
                }
            }

            // ---- Nested declarations (inside function bodies) ----------------
            StmtKind::Function(decl) => {
                // Nested named function — analyze its body in the same issue buffer
                // so that undefined-function/class calls inside it are reported.
                let params: Vec<mir_codebase::FnParam> = decl
                    .params
                    .iter()
                    .map(|p| mir_codebase::FnParam {
                        name: std::sync::Arc::from(p.name.trim_start_matches('$')),
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
                    self.codebase,
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

            StmtKind::Class(decl) => {
                // Nested class declaration — analyze each method body in the same
                // issue buffer so that undefined-function/class calls are reported.
                let class_name = decl.name.unwrap_or("<anonymous>");
                let resolved = self.codebase.resolve_class_name(&self.file, class_name);
                let fqcn: Arc<str> = Arc::from(resolved.as_str());
                let parent_fqcn = self
                    .codebase
                    .classes
                    .get(fqcn.as_ref())
                    .and_then(|c| c.parent.clone());

                for member in decl.members.iter() {
                    let php_ast::ast::ClassMemberKind::Method(method) = &member.kind else {
                        continue;
                    };
                    let Some(body) = &method.body else { continue };
                    let (params, return_ty) = self
                        .codebase
                        .get_method(fqcn.as_ref(), method.name)
                        .as_deref()
                        .map(|m| (m.params.clone(), m.return_type.clone()))
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
                        self.codebase,
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

            StmtKind::Interface(_) | StmtKind::Trait(_) | StmtKind::Enum(_) => {
                // Interfaces/traits/enums are collected in Pass 1 — skip here
            }

            // ---- Namespace / use (at file level, already handled in Pass 1) --
            StmtKind::Namespace(_) | StmtKind::Use(_) | StmtKind::Const(_) => {}

            // ---- Inert --------------------------------------------------------
            StmtKind::InlineHtml(_)
            | StmtKind::Nop
            | StmtKind::Goto(_)
            | StmtKind::Label(_)
            | StmtKind::HaltCompiler(_) => {}

            StmtKind::Error => {}
        }
    }

    // -----------------------------------------------------------------------
    // Helper: create a short-lived ExpressionAnalyzer borrowing our fields
    // -----------------------------------------------------------------------

    fn expr_analyzer<'b>(&'b mut self, _ctx: &Context) -> ExpressionAnalyzer<'b>
    where
        'a: 'b,
    {
        ExpressionAnalyzer::new(
            self.codebase,
            self.file.clone(),
            self.source,
            self.source_map,
            self.issues,
            self.symbols,
            self.php_version,
            self.inference_only,
        )
    }

    /// Convert a byte offset to a Unicode char-count column on a given line.
    /// Returns (line, col) where col is a 0-based Unicode code-point count.
    fn offset_to_line_col(&self, offset: u32) -> (u32, u16) {
        let lc = self.source_map.offset_to_line_col(offset);
        let line = lc.line + 1;

        let byte_offset = offset as usize;
        let line_start_byte = if byte_offset == 0 {
            0
        } else {
            self.source[..byte_offset]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0)
        };

        let col = self.source[line_start_byte..byte_offset].chars().count() as u16;

        (line, col)
    }

    /// Emit `UndefinedClass` for a `Name` AST node if the resolved class does not exist.
    fn check_name_undefined_class(&mut self, name: &php_ast::ast::Name<'_, '_>) {
        let raw = crate::parser::name_to_string(name);
        let resolved = self.codebase.resolve_class_name(&self.file, &raw);
        if matches!(resolved.as_str(), "self" | "static" | "parent") {
            return;
        }
        if self.codebase.type_exists(&resolved) {
            return;
        }
        let span = name.span();
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (line_end, col_end) = self.offset_to_line_col(span.end);
        self.issues.add(Issue::new(
            IssueKind::UndefinedClass { name: resolved },
            Location {
                file: self.file.clone(),
                line,
                line_end,
                col_start,
                col_end: col_end.max(col_start + 1),
            },
        ));
    }

    // -----------------------------------------------------------------------
    // @psalm-suppress / @suppress per-statement
    // -----------------------------------------------------------------------

    /// Extract suppression names from the `@psalm-suppress` / `@suppress`
    /// annotation in the docblock immediately preceding `span`.
    fn extract_statement_suppressions(&self, span: php_ast::Span) -> Vec<String> {
        let Some(doc) = crate::parser::find_preceding_docblock(self.source, span.start) else {
            return vec![];
        };
        let mut suppressions = Vec::new();
        for line in doc.lines() {
            let line = line.trim().trim_start_matches('*').trim();
            let rest = if let Some(r) = line.strip_prefix("@psalm-suppress ") {
                r
            } else if let Some(r) = line.strip_prefix("@suppress ") {
                r
            } else {
                continue;
            };
            for name in rest.split_whitespace() {
                suppressions.push(name.to_string());
            }
        }
        suppressions
    }

    /// Extract `@var Type [$varname]` from the docblock immediately preceding `span`.
    /// Returns `(optional_var_name, resolved_type)` if an annotation exists.
    /// The type is resolved through the codebase's file-level imports/namespace.
    fn extract_var_annotation(
        &self,
        span: php_ast::Span,
    ) -> Option<(Option<String>, mir_types::Union)> {
        let doc = crate::parser::find_preceding_docblock(self.source, span.start)?;
        let parsed = crate::parser::DocblockParser::parse(&doc);
        let ty = parsed.var_type?;
        let resolved = resolve_union_for_file(ty, self.codebase, &self.file);
        Some((parsed.var_name, resolved))
    }

    // -----------------------------------------------------------------------
    // Fixed-point loop widening (M12)
    // -----------------------------------------------------------------------

    /// Analyse a loop body with a fixed-point widening algorithm (≤ 3 passes).
    ///
    /// * `pre`   — context *before* the loop (used as the merge base)
    /// * `entry` — context on first iteration entry (may be narrowed / seeded)
    /// * `body`  — closure that analyses one loop iteration, receives `&mut Self`
    ///   and `&mut Context` for the current iteration context
    ///
    /// Returns the post-loop context that merges:
    ///   - the stable widened context after normal loop exit
    ///   - any contexts captured at `break` statements
    fn analyze_loop_widened<F>(&mut self, pre: &Context, entry: Context, mut body: F) -> Context
    where
        F: FnMut(&mut Self, &mut Context),
    {
        const MAX_ITERS: usize = 3;

        // Push a fresh break-context bucket for this loop level
        self.break_ctx_stack.push(Vec::new());

        let mut current = entry;
        current.inside_loop = true;

        for _ in 0..MAX_ITERS {
            let prev_vars = current.vars.clone();

            let mut iter = current.clone();
            body(self, &mut iter);

            let next = Context::merge_branches(pre, iter, None);

            if vars_stabilized(&prev_vars, &next.vars) {
                current = next;
                break;
            }
            current = next;
        }

        // Widen any variable still unstable after MAX_ITERS to `mixed`
        widen_unstable(&pre.vars, &mut current.vars);

        // Pop break contexts and merge them into the post-loop result
        let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();
        for bctx in break_ctxs {
            current = Context::merge_branches(pre, current, Some(bctx));
        }

        current
    }
}
