/// Statement analyzer — walks statement nodes threading context through
/// control flow (if/else, loops, try/catch, return).
use std::sync::Arc;

use php_ast::ast::StmtKind;

use mir_codebase::Codebase;
use mir_issues::{IssueBuffer, IssueKind};
use mir_types::{Atomic, Union};

use crate::context::Context;
use crate::expr::ExpressionAnalyzer;
use crate::narrowing::narrow_from_condition;
use crate::symbol::ResolvedSymbol;

// ---------------------------------------------------------------------------
// StatementsAnalyzer
// ---------------------------------------------------------------------------

pub struct StatementsAnalyzer<'a> {
    pub codebase: &'a Codebase,
    pub file: Arc<str>,
    pub source: &'a str,
    pub source_map: &'a php_ast::source_map::SourceMap,
    pub issues: &'a mut IssueBuffer,
    pub symbols: &'a mut Vec<ResolvedSymbol>,
    /// Accumulated inferred return types for the current function.
    pub return_types: Vec<Union>,
    /// Break-context stack: one entry per active loop nesting level.
    /// Each entry collects the context states at every `break` in that loop.
    break_ctx_stack: Vec<Vec<Context>>,
}

impl<'a> StatementsAnalyzer<'a> {
    pub fn new(
        codebase: &'a Codebase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_ast::source_map::SourceMap,
        issues: &'a mut IssueBuffer,
        symbols: &'a mut Vec<ResolvedSymbol>,
    ) -> Self {
        Self {
            codebase,
            file,
            source,
            source_map,
            issues,
            symbols,
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
                self.expr_analyzer(ctx).analyze(expr, ctx);
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
                        let lc = self.source_map.offset_to_line_col(stmt.span.start);
                        let (line, col) = (lc.line + 1, lc.col as u16);
                        let mut issue = mir_issues::Issue::new(
                            IssueKind::TaintedHtml,
                            mir_issues::Location {
                                file: self.file.clone(),
                                line,
                                col_start: col,
                                col_end: col,
                            },
                        );
                        // Extract snippet from the echo statement span
                        let s = stmt.span.start as usize;
                        let e = (stmt.span.end as usize).min(self.source.len());
                        if let Some(text) = self.source.get(s..e) {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                issue = issue.with_snippet(trimmed.to_string());
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
                            let (line, col) = {
                                let lc = self.source_map.offset_to_line_col(stmt.span.start);
                                (lc.line + 1, lc.col as u16)
                            };
                            self.issues.add(
                                mir_issues::Issue::new(
                                    IssueKind::InvalidReturnType {
                                        expected: format!("{}", declared),
                                        actual: format!("{}", ret_ty),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        col_start: col,
                                        col_end: col,
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
                            let (line, col) = {
                                let lc = self.source_map.offset_to_line_col(stmt.span.start);
                                (lc.line + 1, lc.col as u16)
                            };
                            self.issues.add(
                                mir_issues::Issue::new(
                                    IssueKind::InvalidReturnType {
                                        expected: format!("{}", declared),
                                        actual: "void".to_string(),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        col_start: col,
                                        col_end: col,
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
                                let (line, col) = {
                                    let lc = self.source_map.offset_to_line_col(stmt.span.start);
                                    (lc.line + 1, lc.col as u16)
                                };
                                self.issues.add(mir_issues::Issue::new(
                                    IssueKind::InvalidThrow {
                                        ty: fqcn.to_string(),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        col_start: col,
                                        col_end: col,
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
                                let (line, col) = {
                                    let lc = self.source_map.offset_to_line_col(stmt.span.start);
                                    (lc.line + 1, lc.col as u16)
                                };
                                self.issues.add(mir_issues::Issue::new(
                                    IssueKind::InvalidThrow {
                                        ty: fqcn.to_string(),
                                    },
                                    mir_issues::Location {
                                        file: self.file.clone(),
                                        line,
                                        col_start: col,
                                        col_end: col,
                                    },
                                ));
                            }
                        }
                        mir_types::Atomic::TMixed | mir_types::Atomic::TObject => {}
                        _ => {
                            let (line, col) = {
                                let lc = self.source_map.offset_to_line_col(stmt.span.start);
                                (lc.line + 1, lc.col as u16)
                            };
                            self.issues.add(mir_issues::Issue::new(
                                IssueKind::InvalidThrow {
                                    ty: format!("{}", thrown_ty),
                                },
                                mir_issues::Location {
                                    file: self.file.clone(),
                                    line,
                                    col_start: col,
                                    col_end: col,
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
                // Skip analyzing a statically-unreachable branch (prevents false
                // positives in dead branches caused by overly conservative types).
                if !then_ctx.diverges {
                    self.analyze_stmt(if_stmt.then_branch, &mut then_ctx);
                }

                // ElseIf branches (flatten into separate else-if chain)
                let mut elseif_ctxs: Vec<Context> = vec![];
                for elseif in if_stmt.elseif_branches.iter() {
                    let mut branch_ctx = ctx.fork();
                    narrow_from_condition(
                        &elseif.condition,
                        &mut branch_ctx,
                        true,
                        self.codebase,
                        &self.file,
                    );
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
                if !else_ctx.diverges {
                    if let Some(else_branch) = &if_stmt.else_branch {
                        self.analyze_stmt(else_branch, &mut else_ctx);
                    }
                }

                // Emit RedundantCondition if narrowing proves one branch is statically unreachable.
                if !pre_diverges && (then_ctx.diverges || else_ctx.diverges) {
                    let lc = self
                        .source_map
                        .offset_to_line_col(if_stmt.condition.span.start);
                    let (line, col) = (lc.line + 1, lc.col as u16);
                    self.issues.add(
                        mir_issues::Issue::new(
                            IssueKind::RedundantCondition {
                                ty: format!("{}", cond_type),
                            },
                            mir_issues::Location {
                                file: self.file.clone(),
                                line,
                                col_start: col,
                                col_end: col,
                            },
                        )
                        .with_snippet(
                            crate::parser::span_text(self.source, if_stmt.condition.span)
                                .unwrap_or_default(),
                        ),
                    );
                }

                // Merge all branches
                *ctx = Context::merge_branches(&pre_ctx, then_ctx, Some(else_ctx));
                for ec in elseif_ctxs {
                    *ctx = Context::merge_branches(&pre_ctx, ec, None);
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
                        Some(name.as_ref().trim_start_matches('$').to_string())
                    }
                    _ => None,
                };
                // Detect `switch(true)` — case conditions are used as narrowing expressions
                let switch_on_true = matches!(&sw.expr.kind, php_ast::ast::ExprKind::Bool(true));

                let pre_ctx = ctx.clone();
                // Push a break-context bucket so that `break` inside cases saves
                // the case's context for merging into the post-switch result.
                self.break_ctx_stack.push(Vec::new());

                let mut all_cases_diverge = true;
                let has_default = sw.cases.iter().any(|c| c.value.is_none());

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
                    for stmt in case.body.iter() {
                        self.analyze_stmt(stmt, &mut case_ctx);
                    }
                    if !case_ctx.diverges {
                        all_cases_diverge = false;
                    }
                }

                // Pop break contexts — each `break` in a case body pushed its
                // context here, representing that case's effect on post-switch state.
                let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();

                // Build the post-switch merged context:
                // Start with pre_ctx if no default case (switch might not match anything)
                // or if not all cases diverge via return/throw.
                let mut merged = if has_default && all_cases_diverge && break_ctxs.is_empty() {
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

                *ctx = merged;
            }

            // ---- Try/catch/finally -------------------------------------------
            StmtKind::TryCatch(tc) => {
                let pre_ctx = ctx.clone();
                let mut try_ctx = ctx.fork();
                for stmt in tc.body.iter() {
                    self.analyze_stmt(stmt, &mut try_ctx);
                }

                // Build a base context for catch blocks that merges pre and try contexts.
                // Variables that might have been set during the try body are "possibly assigned"
                // in the catch (they may or may not have been set before the exception fired).
                let catch_base = Context::merge_branches(&pre_ctx, try_ctx.clone(), None);

                let mut non_diverging_catches: Vec<Context> = vec![];
                for catch in tc.catches.iter() {
                    let mut catch_ctx = catch_base.clone();
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
                    for stmt in catch.body.iter() {
                        self.analyze_stmt(stmt, &mut catch_ctx);
                    }
                    if !catch_ctx.diverges {
                        non_diverging_catches.push(catch_ctx);
                    }
                }

                // If ALL catch branches diverge (return/throw/continue/break),
                // code after the try/catch is only reachable from the try body.
                // Use try_ctx directly so variables assigned in try are definitely set.
                let result = if non_diverging_catches.is_empty() {
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
                    for stmt in finally_stmts.iter() {
                        self.analyze_stmt(stmt, &mut finally_ctx);
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
                        ctx.unset_var(name.as_ref().trim_start_matches('$'));
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
                        ctx.set_var(name.as_ref().trim_start_matches('$'), Union::mixed());
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
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_)
            | StmtKind::Enum(_) => {
                // Nested declarations are collected in Pass 1 — skip here
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
        )
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

// ---------------------------------------------------------------------------
// Loop widening helpers
// ---------------------------------------------------------------------------

/// Returns true when every variable present in `prev` has the same type in
/// `next`, indicating the fixed-point has been reached.
fn vars_stabilized(
    prev: &indexmap::IndexMap<String, Union>,
    next: &indexmap::IndexMap<String, Union>,
) -> bool {
    if prev.len() != next.len() {
        return false;
    }
    prev.iter()
        .all(|(k, v)| next.get(k).map(|u| u == v).unwrap_or(false))
}

/// For any variable whose type changed relative to `pre_vars`, widen to
/// `mixed`.  Called after MAX_ITERS to avoid non-termination.
fn widen_unstable(
    pre_vars: &indexmap::IndexMap<String, Union>,
    current_vars: &mut indexmap::IndexMap<String, Union>,
) {
    for (name, ty) in current_vars.iter_mut() {
        if pre_vars.get(name).map(|p| p != ty).unwrap_or(true) && !ty.is_mixed() {
            *ty = Union::mixed();
        }
    }
}

// ---------------------------------------------------------------------------
// foreach key/value type inference
// ---------------------------------------------------------------------------

fn infer_foreach_types(arr_ty: &Union) -> (Union, Union) {
    if arr_ty.is_mixed() {
        return (Union::mixed(), Union::mixed());
    }
    for atomic in &arr_ty.types {
        match atomic {
            Atomic::TArray { key, value } | Atomic::TNonEmptyArray { key, value } => {
                return (*key.clone(), *value.clone());
            }
            Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                return (Union::single(Atomic::TInt), *value.clone());
            }
            Atomic::TKeyedArray { properties, .. } => {
                let mut values = Union::empty();
                for (_k, prop) in properties {
                    values = Union::merge(&values, &prop.ty);
                }
                // Empty keyed array (e.g. `$arr = []` before push) — treat value as mixed
                // to avoid propagating Union::empty() as a variable type.
                let values = if values.is_empty() {
                    Union::mixed()
                } else {
                    values
                };
                return (Union::single(Atomic::TMixed), values);
            }
            Atomic::TString => {
                return (Union::single(Atomic::TInt), Union::single(Atomic::TString));
            }
            _ => {}
        }
    }
    (Union::mixed(), Union::mixed())
}

// ---------------------------------------------------------------------------
// Named-object return type compatibility check
// ---------------------------------------------------------------------------

/// Returns true if `actual` is compatible with `declared` considering class
/// hierarchy, self/static resolution, and short-name vs FQCN mismatches.
fn named_object_return_compatible(
    actual: &Union,
    declared: &Union,
    codebase: &Codebase,
    file: &str,
) -> bool {
    actual.types.iter().all(|actual_atom| {
        // Extract the actual FQCN — handles TNamedObject, TSelf, TStaticObject, TParent
        let actual_fqcn: &Arc<str> = match actual_atom {
            Atomic::TNamedObject { fqcn, .. } => fqcn,
            Atomic::TSelf { fqcn } => fqcn,
            Atomic::TStaticObject { fqcn } => fqcn,
            Atomic::TParent { fqcn } => fqcn,
            // TNull: compatible if declared also includes null
            Atomic::TNull => return declared.types.iter().any(|d| matches!(d, Atomic::TNull)),
            // TVoid: compatible with void declared
            Atomic::TVoid => {
                return declared
                    .types
                    .iter()
                    .any(|d| matches!(d, Atomic::TVoid | Atomic::TNull))
            }
            // TNever is the bottom type — compatible with anything
            Atomic::TNever => return true,
            // class-string<X> is compatible with class-string<Y> if X extends/implements Y
            Atomic::TClassString(Some(actual_cls)) => {
                return declared.types.iter().any(|d| match d {
                    Atomic::TClassString(None) => true,
                    Atomic::TClassString(Some(declared_cls)) => {
                        actual_cls == declared_cls
                            || codebase
                                .extends_or_implements(actual_cls.as_ref(), declared_cls.as_ref())
                    }
                    Atomic::TString => true,
                    _ => false,
                });
            }
            Atomic::TClassString(None) => {
                return declared
                    .types
                    .iter()
                    .any(|d| matches!(d, Atomic::TClassString(_) | Atomic::TString));
            }
            // Non-object types: not handled here (fall through to simple subtype check)
            _ => return false,
        };

        declared.types.iter().any(|declared_atom| {
            // Extract declared FQCN — also handle self/static/parent in declared type
            let declared_fqcn: &Arc<str> = match declared_atom {
                Atomic::TNamedObject { fqcn, .. } => fqcn,
                Atomic::TSelf { fqcn } => fqcn,
                Atomic::TStaticObject { fqcn } => fqcn,
                Atomic::TParent { fqcn } => fqcn,
                _ => return false,
            };

            let resolved_declared = codebase.resolve_class_name(file, declared_fqcn.as_ref());
            let resolved_actual = codebase.resolve_class_name(file, actual_fqcn.as_ref());

            // Self/static always compatible with the class itself
            if matches!(
                actual_atom,
                Atomic::TSelf { .. } | Atomic::TStaticObject { .. }
            ) && (resolved_actual == resolved_declared
                    || actual_fqcn.as_ref() == declared_fqcn.as_ref()
                    || actual_fqcn.as_ref() == resolved_declared.as_str()
                    || resolved_actual.as_str() == declared_fqcn.as_ref()
                    || codebase.extends_or_implements(actual_fqcn.as_ref(), &resolved_declared)
                    || codebase.extends_or_implements(actual_fqcn.as_ref(), declared_fqcn.as_ref())
                    || codebase.extends_or_implements(&resolved_actual, &resolved_declared)
                    || codebase.extends_or_implements(&resolved_actual, declared_fqcn.as_ref())
                    // static(X) is compatible with declared Y if Y extends X
                    // (because when called on Y, static = Y which satisfies declared Y)
                    || codebase.extends_or_implements(&resolved_declared, actual_fqcn.as_ref())
                    || codebase.extends_or_implements(&resolved_declared, &resolved_actual)
                    || codebase.extends_or_implements(declared_fqcn.as_ref(), actual_fqcn.as_ref()))
            {
                return true;
            }

            // Same after resolution
            resolved_actual == resolved_declared
                // Direct string match in any combination
                || actual_fqcn.as_ref() == declared_fqcn.as_ref()
                || actual_fqcn.as_ref() == resolved_declared.as_str()
                || resolved_actual.as_str() == declared_fqcn.as_ref()
                // Inheritance check
                || codebase.extends_or_implements(actual_fqcn.as_ref(), &resolved_declared)
                || codebase.extends_or_implements(actual_fqcn.as_ref(), declared_fqcn.as_ref())
                || codebase.extends_or_implements(&resolved_actual, &resolved_declared)
                || codebase.extends_or_implements(&resolved_actual, declared_fqcn.as_ref())
        })
    })
}

/// Returns true if the declared return type contains template-like types (unknown FQCNs
/// without namespace separator that don't exist in the codebase) — we can't validate
/// return types against generic type parameters without full template instantiation.
fn declared_return_has_template(declared: &Union, codebase: &Codebase) -> bool {
    declared.types.iter().any(|atomic| match atomic {
        Atomic::TTemplateParam { .. } => true,
        // Generic class instantiation (e.g. Result<string, void>) — skip without full template inference.
        // Also skip when the named class doesn't exist in the codebase (e.g. type aliases
        // that were resolved to a fully-qualified name but aren't real classes).
        // Also skip when the type is an interface — concrete implementations may satisfy the
        // declared type in ways we don't track (not flagged at default error level).
        Atomic::TNamedObject { fqcn, type_params } => {
            !type_params.is_empty()
                || !codebase.type_exists(fqcn.as_ref())
                || codebase.interfaces.contains_key(fqcn.as_ref())
        }
        Atomic::TArray { value, .. }
        | Atomic::TList { value }
        | Atomic::TNonEmptyArray { value, .. }
        | Atomic::TNonEmptyList { value } => value.types.iter().any(|v| match v {
            Atomic::TTemplateParam { .. } => true,
            Atomic::TNamedObject { fqcn, .. } => {
                !fqcn.contains('\\') && !codebase.type_exists(fqcn.as_ref())
            }
            _ => false,
        }),
        _ => false,
    })
}

/// Resolve all TNamedObject FQCNs in a Union using the codebase's file-level imports/namespace.
/// Used to fix up `@var` annotation types that were parsed without namespace context.
fn resolve_union_for_file(union: Union, codebase: &Codebase, file: &str) -> Union {
    let mut result = Union::empty();
    result.possibly_undefined = union.possibly_undefined;
    result.from_docblock = union.from_docblock;
    for atomic in union.types {
        let resolved = resolve_atomic_for_file(atomic, codebase, file);
        result.types.push(resolved);
    }
    result
}

fn is_resolvable_class_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '\\')
}

fn resolve_atomic_for_file(atomic: Atomic, codebase: &Codebase, file: &str) -> Atomic {
    match atomic {
        Atomic::TNamedObject { fqcn, type_params } => {
            if !is_resolvable_class_name(fqcn.as_ref()) {
                return Atomic::TNamedObject { fqcn, type_params };
            }
            let resolved = codebase.resolve_class_name(file, fqcn.as_ref());
            Atomic::TNamedObject {
                fqcn: resolved.into(),
                type_params,
            }
        }
        Atomic::TClassString(Some(cls)) => {
            let resolved = codebase.resolve_class_name(file, cls.as_ref());
            Atomic::TClassString(Some(resolved.into()))
        }
        Atomic::TList { value } => Atomic::TList {
            value: Box::new(resolve_union_for_file(*value, codebase, file)),
        },
        Atomic::TNonEmptyList { value } => Atomic::TNonEmptyList {
            value: Box::new(resolve_union_for_file(*value, codebase, file)),
        },
        Atomic::TArray { key, value } => Atomic::TArray {
            key: Box::new(resolve_union_for_file(*key, codebase, file)),
            value: Box::new(resolve_union_for_file(*value, codebase, file)),
        },
        Atomic::TSelf { fqcn } if fqcn.is_empty() => {
            // Sentinel from docblock parser — leave as-is; caller handles it
            Atomic::TSelf { fqcn }
        }
        other => other,
    }
}

/// Returns true if both actual and declared are array/list types whose value types are
/// compatible with FQCN resolution (to avoid short-name vs FQCN mismatches in return types).
fn return_arrays_compatible(
    actual: &Union,
    declared: &Union,
    codebase: &Codebase,
    file: &str,
) -> bool {
    actual.types.iter().all(|a_atomic| {
        let act_val: &Union = match a_atomic {
            Atomic::TArray { value, .. }
            | Atomic::TNonEmptyArray { value, .. }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => value,
            Atomic::TKeyedArray { .. } => return true,
            _ => return false,
        };

        declared.types.iter().any(|d_atomic| {
            let dec_val: &Union = match d_atomic {
                Atomic::TArray { value, .. }
                | Atomic::TNonEmptyArray { value, .. }
                | Atomic::TList { value }
                | Atomic::TNonEmptyList { value } => value,
                _ => return false,
            };

            act_val.types.iter().all(|av| {
                match av {
                    Atomic::TNever => return true,
                    Atomic::TClassString(Some(av_cls)) => {
                        return dec_val.types.iter().any(|dv| match dv {
                            Atomic::TClassString(None) | Atomic::TString => true,
                            Atomic::TClassString(Some(dv_cls)) => {
                                av_cls == dv_cls
                                    || codebase
                                        .extends_or_implements(av_cls.as_ref(), dv_cls.as_ref())
                            }
                            _ => false,
                        });
                    }
                    _ => {}
                }
                let av_fqcn: &Arc<str> = match av {
                    Atomic::TNamedObject { fqcn, .. } => fqcn,
                    Atomic::TSelf { fqcn } | Atomic::TStaticObject { fqcn } => fqcn,
                    Atomic::TClosure { .. } => return true,
                    _ => return Union::single(av.clone()).is_subtype_of_simple(dec_val),
                };
                dec_val.types.iter().any(|dv| {
                    let dv_fqcn: &Arc<str> = match dv {
                        Atomic::TNamedObject { fqcn, .. } => fqcn,
                        Atomic::TClosure { .. } => return true,
                        _ => return false,
                    };
                    if !dv_fqcn.contains('\\') && !codebase.type_exists(dv_fqcn.as_ref()) {
                        return true; // template param wildcard
                    }
                    let res_dec = codebase.resolve_class_name(file, dv_fqcn.as_ref());
                    let res_act = codebase.resolve_class_name(file, av_fqcn.as_ref());
                    res_dec == res_act
                        || codebase.extends_or_implements(av_fqcn.as_ref(), &res_dec)
                        || codebase.extends_or_implements(&res_act, &res_dec)
                })
            })
        })
    })
}
