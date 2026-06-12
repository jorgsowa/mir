use std::sync::Arc;

use php_ast::owned::{
    DoWhileStmt, ExprKind, ForStmt, ForeachStmt, IfStmt, SwitchStmt, TryCatchStmt, WhileStmt,
};

use mir_issues::{Issue, IssueKind, Location};
use mir_types::{Atomic, Name, Type};

use crate::db;
use crate::expr::{extract_destructure_vars, extract_simple_var};
use crate::flow_state::FlowState;
use crate::narrowing::narrow_from_condition;
use crate::parser;

use super::loops::infer_foreach_types;
use super::StatementsAnalyzer;

impl<'a> StatementsAnalyzer<'a> {
    pub(super) fn analyze_if_stmt(&mut self, if_stmt: &IfStmt, ctx: &mut FlowState) {
        let pre_ctx = ctx.clone();

        let cond_type = self.expr_analyzer(ctx).analyze(&if_stmt.condition, ctx);
        let pre_diverges = ctx.diverges;

        let mut then_ctx = ctx.branch();
        narrow_from_condition(&if_stmt.condition, &mut then_ctx, true, self.db, &self.file);
        let then_unreachable_from_narrowing = then_ctx.diverges;
        if !then_ctx.diverges {
            self.analyze_stmt(&if_stmt.then_branch, &mut then_ctx);
        }

        let mut elseif_ctxs: Vec<FlowState> = vec![];
        for elseif in if_stmt.elseif_branches.iter() {
            let mut pre_elseif = ctx.branch();
            narrow_from_condition(
                &if_stmt.condition,
                &mut pre_elseif,
                false,
                self.db,
                &self.file,
            );
            let pre_elseif_diverges = pre_elseif.diverges;

            let mut elseif_true_ctx = pre_elseif.clone();
            narrow_from_condition(
                &elseif.condition,
                &mut elseif_true_ctx,
                true,
                self.db,
                &self.file,
            );
            let mut elseif_false_ctx = pre_elseif.clone();
            narrow_from_condition(
                &elseif.condition,
                &mut elseif_false_ctx,
                false,
                self.db,
                &self.file,
            );
            if !pre_elseif_diverges && (elseif_true_ctx.diverges || elseif_false_ctx.diverges) {
                let (line, line_end, col_start, col_end) =
                    self.span_to_location(elseif.condition.span);
                let elseif_cond_type = self
                    .expr_analyzer(ctx)
                    .analyze(&elseif.condition, &mut ctx.branch());
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
                        parser::span_text(self.source, elseif.condition.span).unwrap_or_default(),
                    ),
                );
            }

            let mut branch_ctx = elseif_true_ctx;
            self.expr_analyzer(&branch_ctx)
                .analyze(&elseif.condition, &mut branch_ctx);
            if !branch_ctx.diverges {
                self.analyze_stmt(&elseif.body, &mut branch_ctx);
            }
            elseif_ctxs.push(branch_ctx);
        }

        let mut else_ctx = ctx.branch();
        // For `if ($x = expr())`, in the false branch the assignment was evaluated
        // and found falsy — the write is consumed by the truthiness check. Remove
        // the pending-write entry so that using $x only in the true branch does not
        // trigger UnusedVariable.
        {
            let cond = match &if_stmt.condition.kind {
                ExprKind::Parenthesized(inner) => inner.as_ref(),
                _ => &if_stmt.condition,
            };
            if let ExprKind::Assign(a) = &cond.kind {
                if let Some(var_name) = extract_simple_var(&a.target) {
                    else_ctx
                        .last_write_locs
                        .remove(&Name::from(var_name.as_str()));
                }
            }
        }
        narrow_from_condition(
            &if_stmt.condition,
            &mut else_ctx,
            false,
            self.db,
            &self.file,
        );
        // Redundancy of the outer if condition depends only on its own narrowing, not elseifs.
        let else_unreachable_from_narrowing = else_ctx.diverges;
        // In the else branch all elseif conditions also failed — narrow them out for better
        // type accuracy in the else body (e.g. string|array|int becomes int after is_string
        // and is_array elseifs). Only applied when the else is itself reachable.
        if !else_ctx.diverges {
            for elseif in if_stmt.elseif_branches.iter() {
                if !else_ctx.diverges {
                    narrow_from_condition(
                        &elseif.condition,
                        &mut else_ctx,
                        false,
                        self.db,
                        &self.file,
                    );
                }
            }
        }
        if !else_ctx.diverges {
            if let Some(else_branch) = &if_stmt.else_branch {
                self.analyze_stmt(else_branch, &mut else_ctx);
            }
        }

        if !pre_diverges && (then_unreachable_from_narrowing || else_unreachable_from_narrowing) {
            let (line, line_end, col_start, col_end) =
                self.span_to_location(if_stmt.condition.span);
            self.issues.add(
                Issue::new(
                    IssueKind::RedundantCondition {
                        ty: format!("{cond_type}"),
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
                    parser::span_text(self.source, if_stmt.condition.span).unwrap_or_default(),
                ),
            );
        }

        *ctx = FlowState::merge_branches(&pre_ctx, then_ctx, Some(else_ctx));
        for ec in elseif_ctxs {
            *ctx = FlowState::merge_branches(&pre_ctx, ec, Some(ctx.clone()));
        }
    }

    pub(super) fn analyze_while_stmt(&mut self, w: &WhileStmt, ctx: &mut FlowState) {
        self.expr_analyzer(ctx).analyze(&w.condition, ctx);
        let pre = ctx.clone();

        let mut entry = ctx.branch();
        narrow_from_condition(&w.condition, &mut entry, true, self.db, &self.file);

        let is_infinite = matches!(w.condition.kind, ExprKind::Bool(true));
        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                sa.analyze_stmt(&w.body, iter);
                sa.expr_analyzer(iter).analyze(&w.condition, iter);
            },
            is_infinite,
            is_infinite,
        );
        *ctx = post;
    }

    pub(super) fn analyze_dowhile_stmt(&mut self, dw: &DoWhileStmt, ctx: &mut FlowState) {
        let pre = ctx.clone();
        let entry = ctx.branch();
        // Do-while always executes at least once (body before condition check)
        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                sa.analyze_stmt(&dw.body, iter);
                sa.expr_analyzer(iter).analyze(&dw.condition, iter);
            },
            true,
            false,
        );
        *ctx = post;
    }

    pub(super) fn analyze_for_stmt(&mut self, f: &ForStmt, ctx: &mut FlowState) {
        for init in f.init.iter() {
            self.expr_analyzer(ctx).analyze(init, ctx);
        }
        let pre = ctx.clone();
        let mut entry = ctx.branch();
        for cond in f.condition.iter() {
            self.expr_analyzer(&entry).analyze(cond, &mut entry);
        }

        let is_infinite = f.condition.is_empty();
        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                sa.analyze_stmt(&f.body, iter);
                for update in f.update.iter() {
                    sa.expr_analyzer(iter).analyze(update, iter);
                }
                for cond in f.condition.iter() {
                    sa.expr_analyzer(iter).analyze(cond, iter);
                }
            },
            is_infinite,
            is_infinite,
        );
        *ctx = post;
    }

    pub(super) fn analyze_foreach_stmt(
        &mut self,
        fe: &ForeachStmt,
        stmt: &php_ast::owned::Stmt,
        ctx: &mut FlowState,
    ) {
        let arr_ty = self.expr_analyzer(ctx).analyze(&fe.expr, ctx);
        let (key_ty, mut value_ty) = infer_foreach_types(&arr_ty);

        if let Some(vname) = extract_simple_var(&fe.value) {
            let doc = crate::parser::find_preceding_docblock(self.source, stmt.span.start);
            if let Some(ann) = self.extract_var_annotation_from(doc.as_deref()) {
                if ann.name.as_deref() == Some(vname.as_str()) {
                    value_ty = ann.ty;
                }
            }
        }

        let pre = ctx.clone();
        let mut entry = ctx.branch();

        if let Some(key_expr) = &fe.key {
            if let Some(var_name) = extract_simple_var(key_expr) {
                entry.set_var(&var_name, key_ty.clone());
                // Emit ResolvedSymbol for key variable at binding position
                self.record_symbol_for_var(key_expr.span, &var_name, key_ty.clone());
            }
        }
        let value_var = extract_simple_var(&fe.value);
        let value_destructure_vars = extract_destructure_vars(&fe.value);
        if let Some(ref vname) = value_var {
            entry.set_var(vname.as_str(), value_ty.clone());
            // Track this as a foreach value variable so emit_unused_variables can
            // emit UnusedForeachValue instead of UnusedVariable for dead writes.
            entry
                .foreach_value_var_names
                .insert(Name::from(vname.as_str()));
            // Record the header assignment so it appears in last_write_locs and
            // triggers UnusedForeachValue when the value is never read in the body.
            let (line, line_end, col_start, col_end) = self.span_to_location(fe.value.span);
            entry.record_write(vname.as_str(), line, col_start, line_end, col_end);
            // Emit ResolvedSymbol for value variable at binding position
            self.record_symbol_for_var(fe.value.span, vname, value_ty.clone());
            if value_ty.is_mixed() {
                self.issues.add(
                    Issue::new(
                        IssueKind::MixedAssignment { var: vname.clone() },
                        Location {
                            file: self.file.clone(),
                            line,
                            line_end,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        },
                    )
                    .with_snippet(
                        parser::span_text(self.source, fe.value.span).unwrap_or_default(),
                    ),
                );
            }
        } else {
            for vname in &value_destructure_vars {
                entry.set_var(vname, Type::mixed());
            }
        }

        let loop_guaranteed = super::loops::loop_guaranteed_to_execute(&arr_ty);
        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                if let Some(key_expr) = &fe.key {
                    if let Some(var_name) = extract_simple_var(key_expr) {
                        iter.set_var(&var_name, key_ty.clone());
                    }
                }
                if let Some(ref vname) = value_var {
                    iter.set_var(vname.as_str(), value_ty.clone());
                } else {
                    for vname in &value_destructure_vars {
                        iter.set_var(vname, Type::mixed());
                    }
                }
                sa.analyze_stmt(&fe.body, iter);
            },
            loop_guaranteed,
            false,
        );
        *ctx = post;
    }

    /// Emit `ParadoxicalCondition` for `switch` cases whose literal value
    /// repeats an earlier case — the duplicate branch can never be reached
    /// because the first matching case wins. Only literal cases are compared,
    /// so dynamic `case $x:` arms are never flagged.
    fn check_duplicate_case_values(&mut self, sw: &SwitchStmt) {
        let values = sw.body.cases.iter().filter_map(|c| c.value.as_ref());
        for (span, value) in crate::expr::duplicate_literal_conditions(values) {
            let (line, line_end, col_start, col_end) = self.span_to_location(span);
            self.issues.add(
                Issue::new(
                    IssueKind::ParadoxicalCondition { value },
                    Location {
                        file: self.file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: col_end.max(col_start + 1),
                    },
                )
                .with_snippet(parser::span_text(self.source, span).unwrap_or_default()),
            );
        }
    }

    pub(super) fn analyze_switch_stmt(&mut self, sw: &SwitchStmt, ctx: &mut FlowState) {
        self.check_duplicate_case_values(sw);
        let subject_ty = self.expr_analyzer(ctx).analyze(&sw.expr, ctx);
        let subject_var: Option<String> = match &sw.expr.kind {
            ExprKind::Variable(name) => Some(name.trim_start_matches('$').to_string()),
            _ => None,
        };
        let switch_on_true = matches!(&sw.expr.kind, ExprKind::Bool(true));

        let pre_ctx = ctx.clone();
        self.break_ctx_stack.push(Vec::new());

        let has_default = sw.body.cases.iter().any(|c| c.value.is_none());

        let mut case_results: Vec<FlowState> = Vec::new();
        for case in sw.body.cases.iter() {
            let mut case_ctx = pre_ctx.branch();
            if let Some(val) = &case.value {
                if switch_on_true {
                    narrow_from_condition(val, &mut case_ctx, true, self.db, &self.file);
                } else if let Some(ref var_name) = subject_var {
                    let narrow_ty = match &val.kind {
                        ExprKind::Int(n) => Some(Type::single(Atomic::TLiteralInt(*n))),
                        ExprKind::String(s) => {
                            Some(Type::single(Atomic::TLiteralString(Arc::from(&**s))))
                        }
                        ExprKind::Bool(b) => Some(Type::single(if *b {
                            Atomic::TTrue
                        } else {
                            Atomic::TFalse
                        })),
                        ExprKind::Null => Some(Type::single(Atomic::TNull)),
                        _ => None,
                    };
                    if let Some(ref narrowed) = narrow_ty {
                        if !subject_ty.is_mixed() && narrowed.intersect_with(&subject_ty).is_never()
                        {
                            let (line, line_end, col_start, col_end) =
                                self.span_to_location(val.span);
                            self.issues.add(
                                Issue::new(
                                    IssueKind::TypeDoesNotContainType {
                                        left: format!("{subject_ty}"),
                                        right: format!("{narrowed}"),
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
                                    parser::span_text(self.source, val.span).unwrap_or_default(),
                                ),
                            );
                        }
                        case_ctx.set_var(var_name, narrowed.clone());
                    }
                }
                self.expr_analyzer(&case_ctx).analyze(val, &mut case_ctx);
            }
            self.analyze_stmts(&case.body, &mut case_ctx);
            case_results.push(case_ctx);
        }

        let n = case_results.len();
        let mut effective_diverges = vec![false; n];
        for i in (0..n).rev() {
            if case_results[i].diverges {
                effective_diverges[i] = true;
            } else if i + 1 < n {
                effective_diverges[i] = effective_diverges[i + 1];
            }
        }

        let mut fallthrough_ctxs: Vec<FlowState> = Vec::new();
        for (i, case_ctx) in case_results.into_iter().enumerate() {
            if !effective_diverges[i] {
                fallthrough_ctxs.push(case_ctx);
            }
        }

        let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();

        // With a default arm, some arm ALWAYS runs — the "fell past every case"
        // path doesn't exist, so don't seed the merge with pre_ctx (it would
        // keep pre-switch pending writes alive that every arm overwrote).
        let mut merged: Option<FlowState> = if has_default {
            None
        } else {
            Some(pre_ctx.clone())
        };

        for bctx in break_ctxs.into_iter().chain(fallthrough_ctxs) {
            merged = Some(match merged {
                Some(m) => FlowState::merge_branches(&pre_ctx, bctx, Some(m)),
                None => bctx,
            });
        }

        *ctx = merged.unwrap_or_else(|| {
            // has_default and every arm diverges: code after the switch is
            // unreachable.
            let mut m = pre_ctx.clone();
            m.diverges = true;
            m
        });
    }

    pub(super) fn analyze_trycatch_stmt(&mut self, tc: &TryCatchStmt, ctx: &mut FlowState) {
        let pre_ctx = ctx.clone();
        let mut try_ctx = ctx.branch();
        self.analyze_stmts(&tc.body.stmts, &mut try_ctx);

        let catch_base = FlowState::merge_branches(&pre_ctx, try_ctx.clone(), None);

        let mut non_diverging_catches: Vec<FlowState> = vec![];
        for catch in tc.catches.iter() {
            let mut catch_ctx = catch_base.clone();
            // For dead-write tracking, the catch block starts from the pre-try state:
            // an exception can be thrown at any point in the try body, so we don't know
            // which writes from the try body were committed before the throw.
            // Inheriting try-body last_write_locs would cause false "dead write" reports
            // when a catch block re-assigns variables also written in the try body.
            catch_ctx.last_write_locs = pre_ctx.last_write_locs.clone();
            for catch_ty in catch.types.iter() {
                self.check_name_undefined_class(catch_ty);
                if self.mode == crate::body_analysis::AnalysisMode::Full {
                    let raw = parser::name_to_string_owned(catch_ty);
                    let resolved = db::resolve_name(self.db, &self.file, &raw);
                    if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                        let span = catch_ty.span;
                        let (line, col_start) = self.offset_to_line_col(span.start);
                        let (_, col_end) = self.offset_to_line_col(span.end);
                        self.db.record_reference_location(crate::db::RefLoc {
                            symbol_key: Arc::from(resolved.as_str()),
                            file: self.file.clone(),
                            line,
                            col_start,
                            col_end: col_end.max(col_start + 1),
                        });
                        // Check if the caught type extends Throwable
                        if crate::db::class_exists(self.db, &resolved) {
                            let here = crate::db::Fqcn::from_str(self.db, resolved.as_str());
                            if let Some(class) = crate::db::find_class_like(self.db, here) {
                                let written_short = raw.rsplit('\\').next().unwrap_or(raw.as_str());
                                let canonical_short = class
                                    .fqcn()
                                    .rsplit('\\')
                                    .next()
                                    .unwrap_or(class.fqcn().as_ref());
                                if written_short != canonical_short
                                    && written_short.eq_ignore_ascii_case(canonical_short)
                                {
                                    let (line_end, col_end2) = self.offset_to_line_col(span.end);
                                    self.issues.add(Issue::new(
                                        IssueKind::WrongCaseClass {
                                            used: written_short.to_string(),
                                            canonical: canonical_short.to_string(),
                                        },
                                        Location {
                                            file: self.file.clone(),
                                            line,
                                            line_end,
                                            col_start,
                                            col_end: col_end2.max(col_start + 1),
                                        },
                                    ));
                                }
                            }
                            let is_throwable = resolved == "Throwable"
                                || crate::db::extends_or_implements(
                                    self.db,
                                    &resolved,
                                    "Throwable",
                                );
                            if !is_throwable && !crate::db::has_unknown_ancestor(self.db, &resolved)
                            {
                                let (line_end, col_end2) = self.offset_to_line_col(span.end);
                                self.issues.add(Issue::new(
                                    IssueKind::InvalidCatch {
                                        ty: resolved.clone(),
                                    },
                                    Location {
                                        file: self.file.clone(),
                                        line,
                                        line_end,
                                        col_start,
                                        col_end: col_end2.max(col_start + 1),
                                    },
                                ));
                            }
                        }
                    }
                }
            }
            if let Some(var) = &catch.var {
                let exc_ty = if catch.types.is_empty() {
                    Type::single(Atomic::TObject)
                } else {
                    let mut u = Type::empty();
                    for catch_ty in catch.types.iter() {
                        let raw = parser::name_to_string_owned(catch_ty);
                        let resolved = db::resolve_name(self.db, &self.file, &raw);
                        u.add_type(Atomic::TNamedObject {
                            fqcn: resolved.into(),
                            type_params: mir_types::union::empty_type_params(),
                        });
                    }
                    u
                };
                let var_name = var.trim_start_matches('$');
                catch_ctx.set_var(var_name, exc_ty);
                let (line, col_start) = self.offset_to_line_col(catch.span.start);
                let (line_end, col_end) = self.offset_to_line_col(catch.span.end);
                catch_ctx.record_var_location(var_name, line, col_start, line_end, col_end);
            }
            self.analyze_stmts(&catch.body.stmts, &mut catch_ctx);
            if !catch_ctx.diverges {
                non_diverging_catches.push(catch_ctx);
            }
        }

        let mut result = if non_diverging_catches.is_empty() {
            try_ctx
        } else {
            let mut r = try_ctx;
            for catch_ctx in non_diverging_catches {
                r = FlowState::merge_branches(&pre_ctx, r, Some(catch_ctx));
            }
            r
        };

        if let Some(finally_stmts) = &tc.finally {
            let mut finally_ctx = result.clone();
            finally_ctx.inside_finally = true;
            // finally always executes regardless of whether try/catch diverged
            finally_ctx.diverges = false;
            self.analyze_stmts(&finally_stmts.stmts, &mut finally_ctx);
            if finally_ctx.diverges {
                result.diverges = true;
            }
            // Variables read in the finally block count as used — propagate reads back
            // so that the save-restore pattern (assign before try, restore in finally)
            // is not falsely flagged as UnusedVariable.
            for name in finally_ctx.read_vars.iter() {
                result.read_vars.insert(*name);
            }
        }

        *ctx = result;
    }
}
