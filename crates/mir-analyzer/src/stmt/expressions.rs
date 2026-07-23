use super::StatementsAnalyzer;
use crate::flow_state::FlowState;
use crate::narrowing::narrow_from_condition;
use mir_issues::IssueKind;
use php_ast::owned::{Expr, ExprKind};

impl<'a> StatementsAnalyzer<'a> {
    pub(super) fn analyze_expression_stmt(&mut self, expr: &Expr, ctx: &mut FlowState) {
        let expr_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
        if expr_ty.is_never() {
            ctx.diverges = true;
        }
        // A bare comparison statement (e.g. `$a === "aaa";`) whose result is
        // statically fixed by a docblock-derived type.
        self.check_docblock_contradiction(expr, ctx);
        if let ExprKind::FunctionCall(call) = &expr.kind {
            if let ExprKind::Identifier(fn_name) = &call.name.kind {
                if fn_name
                    .as_ref()
                    .trim_start_matches('\\')
                    .eq_ignore_ascii_case("assert")
                {
                    if let Some(arg) = call.args.first() {
                        // Check the asserted condition *before* narrowing, so the
                        // original (docblock) type is still in scope.
                        self.check_docblock_contradiction(&arg.value, ctx);
                        narrow_from_condition(&arg.value, ctx, true, self.db, &self.file);
                    }
                }
            }
        }
    }

    /// Emit `DocblockTypeContradiction` when `cond` is a comparison that a
    /// docblock-derived type makes impossible.
    pub(super) fn check_docblock_contradiction(&mut self, cond: &Expr, ctx: &mut FlowState) {
        if let Some((expr_repr, declared)) = crate::contradiction::impossible_comparison(cond, ctx)
        {
            self.expr_analyzer(ctx).emit(
                IssueKind::DocblockTypeContradiction {
                    expr: expr_repr,
                    declared,
                },
                mir_issues::Severity::Info,
                cond.span,
            );
        }
    }

    pub(super) fn analyze_echo_stmt(
        &mut self,
        exprs: &[Expr],
        stmt_span: php_ast::Span,
        ctx: &mut FlowState,
    ) {
        // @pure implies no side effects at all — echo is output, a side
        // effect just as much as an impure function call, but had no purity
        // check anywhere. Reuses the existing generic ImpureFunctionCall
        // issue kind (already used for this exact violation shape at
        // call/function.rs), emitted once per statement (not per expr, since
        // `echo $a, $b;` is one statement).
        if ctx.is_in_pure_fn {
            self.expr_analyzer(ctx).emit(
                IssueKind::ImpureFunctionCall {
                    fn_name: "echo".to_string(),
                },
                mir_issues::Severity::Warning,
                stmt_span,
            );
        }
        for expr in exprs.iter() {
            let expr_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
            self.check_echo_implicit_to_string_cast(&expr_ty, expr.span);
            if crate::taint::is_expr_tainted(expr, ctx) {
                let (line, col_start) = self.offset_to_line_col(stmt_span.start);
                let (line_end, col_end) = if stmt_span.start < stmt_span.end {
                    let (end_line, end_col) = self.offset_to_line_col(stmt_span.end);
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
                        col_end: crate::diagnostics::clamp_col_end(
                            line, line_end, col_start, col_end,
                        ),
                    },
                );
                let start = stmt_span.start as usize;
                let end = stmt_span.end as usize;
                if start < self.source.len() {
                    let end = end.min(self.source.len());
                    let span_text = &self.source[start..end];
                    if let Some(first_line) = span_text.lines().next() {
                        issue = issue.with_snippet(first_line.trim().to_string());
                    }
                }
                self.issues.add(issue);
            }
        }
    }

    fn check_echo_implicit_to_string_cast(&mut self, ty: &mir_types::Type, span: php_ast::Span) {
        for atomic in &ty.types {
            // Mirrors expr::binary's implicit-to-string check: a non-Stringable
            // enum is just as fatal to `echo` as any other bare object.
            let fqcn = match atomic {
                mir_types::Atomic::TNamedObject { fqcn, .. } => Some(fqcn),
                mir_types::Atomic::TLiteralEnumCase { enum_fqcn, .. } => Some(enum_fqcn),
                _ => None,
            };
            let Some(fqcn) = fqcn else { continue };
            let fqcn_str = fqcn.as_ref();
            if !crate::db::has_method_in_chain(self.db, fqcn_str, "__toString")
                && !crate::db::extends_or_implements(self.db, fqcn_str, "Stringable")
            {
                let (line, col_start) = self.offset_to_line_col(span.start);
                let (line_end, col_end) = if span.start < span.end {
                    let (end_line, end_col) = self.offset_to_line_col(span.end);
                    (end_line, end_col)
                } else {
                    (line, col_start)
                };
                self.issues.add(mir_issues::Issue::new(
                    IssueKind::ImplicitToStringCast {
                        class: fqcn_str.to_string(),
                    },
                    mir_issues::Location {
                        file: self.file.clone(),
                        line,
                        line_end,
                        col_start,
                        col_end: crate::diagnostics::clamp_col_end(
                            line, line_end, col_start, col_end,
                        ),
                    },
                ));
            }
        }
    }
}
