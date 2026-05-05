/// Statement analyzer — walks statement nodes threading context through
/// control flow (if/else, loops, try/catch, return).
mod control_flow;
mod declarations;
mod expressions;
mod flow;
mod loops;
mod return_type;

use loops::{vars_stabilized, widen_unstable};
pub(crate) use return_type::named_object_return_compatible;
use return_type::resolve_union_for_file;

use std::sync::Arc;

use php_ast::ast::StmtKind;

use mir_issues::{Issue, IssueBuffer, IssueKind, Location};
use mir_types::Union;

use crate::context::Context;
use crate::db::MirDatabase;
use crate::expr::ExpressionAnalyzer;
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

// ---------------------------------------------------------------------------
// StatementsAnalyzer
// ---------------------------------------------------------------------------

pub struct StatementsAnalyzer<'a> {
    pub db: &'a dyn MirDatabase,
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
        db: &'a dyn MirDatabase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
        issues: &'a mut IssueBuffer,
        symbols: &'a mut Vec<ResolvedSymbol>,
        php_version: PhpVersion,
        inference_only: bool,
    ) -> Self {
        Self {
            db,
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
                self.analyze_expression_stmt(expr, ctx);
            }

            // ---- Echo ---------------------------------------------------------
            StmtKind::Echo(exprs) => {
                self.analyze_echo_stmt(exprs, stmt.span, ctx);
            }

            // ---- Return -------------------------------------------------------
            StmtKind::Return(opt_expr) => {
                self.analyze_return_stmt(opt_expr, stmt.span, ctx);
            }

            // ---- Throw --------------------------------------------------------
            StmtKind::Throw(expr) => {
                self.analyze_throw_stmt(expr, stmt.span, ctx);
            }

            // ---- If -----------------------------------------------------------
            StmtKind::If(if_stmt) => {
                self.analyze_if_stmt(if_stmt, ctx);
            }

            // ---- While --------------------------------------------------------
            StmtKind::While(w) => {
                self.analyze_while_stmt(w, ctx);
            }

            // ---- Do-while -----------------------------------------------------
            StmtKind::DoWhile(dw) => {
                self.analyze_dowhile_stmt(dw, ctx);
            }

            // ---- For ----------------------------------------------------------
            StmtKind::For(f) => {
                self.analyze_for_stmt(f, ctx);
            }

            // ---- Foreach ------------------------------------------------------
            StmtKind::Foreach(fe) => {
                self.analyze_foreach_stmt(fe, stmt.span, ctx);
            }

            // ---- Switch -------------------------------------------------------
            StmtKind::Switch(sw) => {
                self.analyze_switch_stmt(sw, ctx);
            }

            // ---- Try/catch/finally -------------------------------------------
            StmtKind::TryCatch(tc) => {
                self.analyze_trycatch_stmt(tc, ctx);
            }

            // ---- Block --------------------------------------------------------
            StmtKind::Block(stmts) => {
                self.analyze_stmts(stmts, ctx);
            }

            // ---- Break --------------------------------------------------------
            StmtKind::Break(_) => {
                self.analyze_break_stmt(ctx);
            }

            // ---- Continue ----------------------------------------------------
            StmtKind::Continue(_) => {
                self.analyze_continue_stmt(ctx);
            }

            // ---- Unset --------------------------------------------------------
            StmtKind::Unset(vars) => {
                self.analyze_unset_stmt(vars, ctx);
            }

            // ---- Static variable declaration ---------------------------------
            StmtKind::StaticVar(vars) => {
                self.analyze_static_var_stmt(vars, ctx);
            }

            // ---- Global declaration ------------------------------------------
            StmtKind::Global(vars) => {
                self.analyze_global_stmt(vars, ctx);
            }

            // ---- Declare -----------------------------------------------------
            StmtKind::Declare(d) => {
                self.analyze_declare_stmt(d, ctx);
            }

            // ---- Nested declarations (inside function bodies) ----------------
            StmtKind::Function(decl) => {
                self.analyze_function_decl_stmt(decl, ctx);
            }

            StmtKind::Class(decl) => {
                self.analyze_class_decl_stmt(decl, ctx);
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
            self.db,
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
        let resolved = crate::db::resolve_name_via_db(self.db, &self.file, &raw);
        if matches!(resolved.as_str(), "self" | "static" | "parent") {
            return;
        }
        if crate::db::type_exists_via_db(self.db, &resolved) {
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
        let resolved = resolve_union_for_file(ty, self.db, &self.file);
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
