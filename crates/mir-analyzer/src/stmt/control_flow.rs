use std::sync::Arc;

use php_ast::ast::ExprKind;

use mir_issues::{Issue, IssueKind, Location};
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::db;
use crate::expr::{extract_destructure_vars, extract_simple_var};
use crate::narrowing::narrow_from_condition;
use crate::parser;

use super::loops::infer_foreach_types;
use super::StatementsAnalyzer;

impl<'a> StatementsAnalyzer<'a> {
    pub(super) fn analyze_if_stmt<'arena, 'src>(
        &mut self,
        if_stmt: &php_ast::ast::IfStmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let pre_ctx = ctx.clone();

        let cond_type = self.expr_analyzer(ctx).analyze(&if_stmt.condition, ctx);
        let pre_diverges = ctx.diverges;

        let mut then_ctx = ctx.fork();
        narrow_from_condition(&if_stmt.condition, &mut then_ctx, true, self.db, &self.file);
        let then_unreachable_from_narrowing = then_ctx.diverges;
        if !then_ctx.diverges {
            self.analyze_stmt(if_stmt.then_branch, &mut then_ctx);
        }

        let mut elseif_ctxs: Vec<Context> = vec![];
        for elseif in if_stmt.elseif_branches.iter() {
            let mut pre_elseif = ctx.fork();
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

        let mut else_ctx = ctx.fork();
        narrow_from_condition(
            &if_stmt.condition,
            &mut else_ctx,
            false,
            self.db,
            &self.file,
        );
        let else_unreachable_from_narrowing = else_ctx.diverges;
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

        *ctx = Context::merge_branches(&pre_ctx, then_ctx, Some(else_ctx));
        for ec in elseif_ctxs {
            *ctx = Context::merge_branches(&pre_ctx, ec, Some(ctx.clone()));
        }
    }

    pub(super) fn analyze_while_stmt<'arena, 'src>(
        &mut self,
        w: &php_ast::ast::WhileStmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        self.expr_analyzer(ctx).analyze(&w.condition, ctx);
        let pre = ctx.clone();

        let mut entry = ctx.fork();
        narrow_from_condition(&w.condition, &mut entry, true, self.db, &self.file);

        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                sa.analyze_stmt(w.body, iter);
                sa.expr_analyzer(iter).analyze(&w.condition, iter);
            },
            false,
        );
        *ctx = post;
    }

    pub(super) fn analyze_dowhile_stmt<'arena, 'src>(
        &mut self,
        dw: &php_ast::ast::DoWhileStmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let pre = ctx.clone();
        let entry = ctx.fork();
        // Do-while always executes at least once (body before condition check)
        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                sa.analyze_stmt(dw.body, iter);
                sa.expr_analyzer(iter).analyze(&dw.condition, iter);
            },
            true,
        );
        *ctx = post;
    }

    pub(super) fn analyze_for_stmt<'arena, 'src>(
        &mut self,
        f: &php_ast::ast::ForStmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        for init in f.init.iter() {
            self.expr_analyzer(ctx).analyze(init, ctx);
        }
        let pre = ctx.clone();
        let mut entry = ctx.fork();
        for cond in f.condition.iter() {
            self.expr_analyzer(&entry).analyze(cond, &mut entry);
        }

        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                sa.analyze_stmt(f.body, iter);
                for update in f.update.iter() {
                    sa.expr_analyzer(iter).analyze(update, iter);
                }
                for cond in f.condition.iter() {
                    sa.expr_analyzer(iter).analyze(cond, iter);
                }
            },
            false,
        );
        *ctx = post;
    }

    pub(super) fn analyze_foreach_stmt<'arena, 'src>(
        &mut self,
        fe: &php_ast::ast::ForeachStmt<'arena, 'src>,
        stmt_span: php_ast::Span,
        ctx: &mut Context,
    ) {
        let arr_ty = self.expr_analyzer(ctx).analyze(&fe.expr, ctx);
        let (key_ty, mut value_ty) = infer_foreach_types(&arr_ty);

        if let Some(vname) = extract_simple_var(&fe.value) {
            if let Some((Some(ann_var), ann_ty)) = self.extract_var_annotation(stmt_span) {
                if ann_var == vname {
                    value_ty = ann_ty;
                }
            }
        }

        let pre = ctx.clone();
        let mut entry = ctx.fork();

        if let Some(key_expr) = &fe.key {
            if let Some(var_name) = extract_simple_var(key_expr) {
                entry.set_var(var_name, key_ty.clone());
            }
        }
        let value_var = extract_simple_var(&fe.value);
        let value_destructure_vars = extract_destructure_vars(&fe.value);
        if let Some(ref vname) = value_var {
            entry.set_var(vname.as_str(), value_ty.clone());
        } else {
            for vname in &value_destructure_vars {
                entry.set_var(vname, Union::mixed());
            }
        }

        let loop_guaranteed = super::loops::loop_guaranteed_to_execute(&arr_ty);
        let post = self.analyze_loop_widened(
            &pre,
            entry,
            |sa, iter| {
                if let Some(key_expr) = &fe.key {
                    if let Some(var_name) = extract_simple_var(key_expr) {
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
            },
            loop_guaranteed,
        );
        *ctx = post;
    }

    pub(super) fn analyze_switch_stmt<'arena, 'src>(
        &mut self,
        sw: &php_ast::ast::SwitchStmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let _subject_ty = self.expr_analyzer(ctx).analyze(&sw.expr, ctx);
        let subject_var: Option<String> = match &sw.expr.kind {
            ExprKind::Variable(name) => Some(name.as_str().trim_start_matches('$').to_string()),
            _ => None,
        };
        let switch_on_true = matches!(&sw.expr.kind, ExprKind::Bool(true));

        let pre_ctx = ctx.clone();
        self.break_ctx_stack.push(Vec::new());

        let has_default = sw.cases.iter().any(|c| c.value.is_none());

        let mut case_results: Vec<Context> = Vec::new();
        for case in sw.cases.iter() {
            let mut case_ctx = pre_ctx.fork();
            if let Some(val) = &case.value {
                if switch_on_true {
                    narrow_from_condition(val, &mut case_ctx, true, self.db, &self.file);
                } else if let Some(ref var_name) = subject_var {
                    let narrow_ty = match &val.kind {
                        ExprKind::Int(n) => Some(Union::single(Atomic::TLiteralInt(*n))),
                        ExprKind::String(s) => {
                            Some(Union::single(Atomic::TLiteralString(Arc::from(&**s))))
                        }
                        ExprKind::Bool(b) => Some(Union::single(if *b {
                            Atomic::TTrue
                        } else {
                            Atomic::TFalse
                        })),
                        ExprKind::Null => Some(Union::single(Atomic::TNull)),
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

        let n = case_results.len();
        let mut effective_diverges = vec![false; n];
        for i in (0..n).rev() {
            if case_results[i].diverges {
                effective_diverges[i] = true;
            } else if i + 1 < n {
                effective_diverges[i] = effective_diverges[i + 1];
            }
        }

        let mut all_cases_diverge = true;
        let mut fallthrough_ctxs: Vec<Context> = Vec::new();
        for (i, case_ctx) in case_results.into_iter().enumerate() {
            if !effective_diverges[i] {
                all_cases_diverge = false;
                fallthrough_ctxs.push(case_ctx);
            }
        }

        let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();

        let mut merged = if has_default
            && all_cases_diverge
            && break_ctxs.is_empty()
            && fallthrough_ctxs.is_empty()
        {
            let mut m = pre_ctx.clone();
            m.diverges = true;
            m
        } else {
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

    pub(super) fn analyze_trycatch_stmt<'arena, 'src>(
        &mut self,
        tc: &php_ast::ast::TryCatchStmt<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let pre_ctx = ctx.clone();
        let mut try_ctx = ctx.fork();
        self.analyze_stmts(&tc.body, &mut try_ctx);

        let catch_base = Context::merge_branches(&pre_ctx, try_ctx.clone(), None);

        let mut non_diverging_catches: Vec<Context> = vec![];
        for catch in tc.catches.iter() {
            let mut catch_ctx = catch_base.clone();
            for catch_ty in catch.types.iter() {
                self.check_name_undefined_class(catch_ty);
            }
            if let Some(var) = catch.var {
                let exc_ty = if catch.types.is_empty() {
                    Union::single(Atomic::TObject)
                } else {
                    let mut u = Union::empty();
                    for catch_ty in catch.types.iter() {
                        let raw = parser::name_to_string(catch_ty);
                        let resolved = db::resolve_name_via_db(self.db, &self.file, &raw);
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

        let mut result = if non_diverging_catches.is_empty() {
            let mut r = try_ctx;
            r.diverges = false;
            r
        } else {
            let mut r = try_ctx;
            for catch_ctx in non_diverging_catches {
                r = Context::merge_branches(&pre_ctx, r, Some(catch_ctx));
            }
            r
        };

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
}
