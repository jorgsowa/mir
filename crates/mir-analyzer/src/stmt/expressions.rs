use super::StatementsAnalyzer;
use crate::context::Context;
use crate::narrowing::narrow_from_condition;
use mir_issues::IssueKind;

impl<'a> StatementsAnalyzer<'a> {
    pub(super) fn analyze_expression_stmt<'arena, 'src>(
        &mut self,
        expr: &php_ast::ast::Expr<'arena, 'src>,
        ctx: &mut Context,
    ) {
        let expr_ty = self.expr_analyzer(ctx).analyze(expr, ctx);
        if expr_ty.is_never() {
            ctx.diverges = true;
        }
        if let php_ast::ast::ExprKind::FunctionCall(call) = &expr.kind {
            if let php_ast::ast::ExprKind::Identifier(fn_name) = &call.name.kind {
                if fn_name.eq_ignore_ascii_case("assert") {
                    if let Some(arg) = call.args.first() {
                        narrow_from_condition(&arg.value, ctx, true, self.db, &self.file);
                    }
                }
            }
        }
    }

    pub(super) fn analyze_echo_stmt<'arena, 'src>(
        &mut self,
        exprs: &php_ast::ast::ArenaVec<'arena, php_ast::ast::Expr<'arena, 'src>>,
        stmt_span: php_ast::Span,
        ctx: &mut Context,
    ) {
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
                        col_end: col_end.max(col_start + 1),
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

    fn check_echo_implicit_to_string_cast(&mut self, ty: &mir_types::Union, span: php_ast::Span) {
        for atomic in &ty.types {
            if let mir_types::Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if crate::db::lookup_method_in_chain(self.db, fqcn_str, "__toString").is_none()
                    && !crate::db::extends_or_implements_via_db(self.db, fqcn_str, "Stringable")
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
                            col_end: col_end.max(col_start + 1),
                        },
                    ));
                }
            }
        }
    }
}
