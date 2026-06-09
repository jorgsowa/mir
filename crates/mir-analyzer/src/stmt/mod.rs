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

use crate::parser::docblock::parse_type_string;

use php_ast::owned::StmtKind;

use mir_issues::{Issue, IssueBuffer, IssueKind, Location};
use mir_types::{Atomic, Type};

use crate::body_analysis::AnalysisMode;
use crate::db::MirDatabase;
use crate::expr::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use crate::php_version::PhpVersion;
use crate::symbol::ResolvedSymbol;

// ---------------------------------------------------------------------------
// VarAnnotation
// ---------------------------------------------------------------------------

/// Parsed `@var` annotation from a docblock preceding a statement.
struct VarAnnotation {
    /// `None` when no `$varname` was given — annotation applies to the statement's LHS.
    name: Option<String>,
    ty: mir_types::Type,
}

/// Apply post-narrow: after `$x = expr()`, override the inferred type with the annotated one.
/// Named `@var Type $x` applies only when the LHS matches. Bare `@var Type` applies to any
/// simple assignment LHS (it annotates the statement, not a specific variable).
fn apply_post_narrow(stmt: &php_ast::owned::Stmt, annotation: &VarAnnotation, ctx: &mut FlowState) {
    let php_ast::owned::StmtKind::Expression(e) = &stmt.kind else {
        return;
    };
    let php_ast::owned::ExprKind::Assign(a) = &e.kind else {
        return;
    };
    if !matches!(&a.op, php_ast::ast::AssignOp::Assign) {
        return;
    }
    let php_ast::owned::ExprKind::Variable(lhs_name) = &a.target.kind else {
        return;
    };
    let lhs = lhs_name.trim_start_matches('$');
    match &annotation.name {
        Some(var_name) => {
            if lhs == var_name.as_str() {
                ctx.set_var(var_name.as_str(), annotation.ty.clone());
            }
        }
        None => {
            ctx.set_var(lhs, annotation.ty.clone());
        }
    }
}

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
    pub mode: AnalysisMode,
    /// Accumulated inferred return types for the current function.
    pub return_types: Vec<Type>,
    /// Break-context stack: one entry per active loop nesting level.
    /// Each entry collects the context states at every `break` in that loop.
    break_ctx_stack: Vec<Vec<FlowState>>,
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
        mode: AnalysisMode,
    ) -> Self {
        Self {
            db,
            file,
            source,
            source_map,
            issues,
            symbols,
            php_version,
            mode,
            return_types: Vec::new(),
            break_ctx_stack: Vec::new(),
        }
    }

    pub fn analyze_stmts(&mut self, stmts: &[php_ast::owned::Stmt], ctx: &mut FlowState) {
        for stmt in stmts.iter() {
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
                break;
            }

            self.analyze_stmt(stmt, ctx);
        }
    }

    pub fn analyze_stmt(&mut self, stmt: &php_ast::owned::Stmt, ctx: &mut FlowState) {
        let doc = crate::parser::find_preceding_docblock(self.source, stmt.span.start);
        let suppressions = self.extract_suppressions_from(doc.as_deref());
        let before = self.issues.issue_count();

        let var_annotation = self.extract_var_annotation_from(doc.as_deref());

        // Pre-narrow: `@var Type $varname` before any statement narrows that variable.
        if let Some(ref ann) = var_annotation {
            if let Some(ref name) = ann.name {
                ctx.set_var(name.as_str(), ann.ty.clone());
            }
        }

        // Check `@mir-check` directives
        let mir_checks = self.extract_mir_checks_from(doc.as_deref());
        let (line, line_end, col_start, col_end) = self.span_to_location(stmt.span);
        for (var_name, expected_str) in mir_checks {
            let expected = parse_type_string(&expected_str);
            let actual = widen_for_check(ctx.get_var(&var_name));
            if expected.to_string() != actual.to_string() {
                self.issues.add(Issue::new(
                    IssueKind::TypeCheckMismatch {
                        var: var_name,
                        expected: expected.to_string(),
                        actual: actual.to_string(),
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

        match &stmt.kind {
            // ---- Expression statement ----------------------------------------
            StmtKind::Expression(expr) => {
                // @template on a closure or arrow function is not valid
                if matches!(
                    expr.kind,
                    php_ast::owned::ExprKind::Closure(_)
                        | php_ast::owned::ExprKind::ArrowFunction(_)
                ) {
                    if let Some(raw_doc) = doc.as_deref() {
                        let parsed = crate::parser::DocblockParser::parse(raw_doc);
                        if !parsed.templates.is_empty() {
                            let lc = self.source_map.offset_to_line_col(stmt.span.start);
                            let line = lc.line + 1;
                            self.issues.add(Issue::new(
                                IssueKind::InvalidDocblock {
                                    message: "@template annotations are not supported on closures or arrow functions".to_string(),
                                },
                                Location {
                                    file: self.file.clone(),
                                    line,
                                    line_end: line,
                                    col_start: 0,
                                    col_end: 0,
                                },
                            ));
                        }
                    }
                }
                self.analyze_expression_stmt(expr, ctx);
            }

            // ---- Echo ---------------------------------------------------------
            StmtKind::Echo(exprs) => {
                self.analyze_echo_stmt(exprs, stmt.span, ctx);
            }

            // ---- Return -------------------------------------------------------
            StmtKind::Return(opt_expr) => {
                self.analyze_return_stmt(opt_expr, stmt, ctx);
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
                self.analyze_foreach_stmt(fe, stmt, ctx);
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
            StmtKind::Block(block) => {
                self.analyze_stmts(&block.stmts, ctx);
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
                // Interfaces/traits/enums are collected in definition collection — skip here
            }

            // ---- Namespace / use (at file level, already handled in definition collection) --
            StmtKind::Namespace(_) | StmtKind::Use(_) | StmtKind::Const(_) => {}

            // ---- Inert --------------------------------------------------------
            StmtKind::InlineHtml(_)
            | StmtKind::Nop
            | StmtKind::Goto(_)
            | StmtKind::Label(_)
            | StmtKind::HaltCompiler(_) => {}

            StmtKind::Error => {}
        }

        // Post-narrow: after `$x = expr()`, override the inferred type if annotated.
        if let Some(ref ann) = var_annotation {
            apply_post_narrow(stmt, ann, ctx);
        }

        if !suppressions.is_empty() {
            self.issues.suppress_range(before, &suppressions);
        }
    }

    // -----------------------------------------------------------------------
    // Helper: create a short-lived ExpressionAnalyzer borrowing our fields
    // -----------------------------------------------------------------------

    pub(crate) fn expr_analyzer<'b>(&'b mut self, ctx: &FlowState) -> ExpressionAnalyzer<'b>
    where
        'a: 'b,
    {
        let mut ea = ExpressionAnalyzer::new(
            self.db,
            self.file.clone(),
            self.source,
            self.source_map,
            self.issues,
            self.symbols,
            self.php_version,
            self.mode,
        );
        ea.strict_types = ctx.strict_types;
        ea
    }

    fn record_symbol_for_var(&mut self, span: php_ast::Span, var_name: &str, ty: Type) {
        use crate::symbol::ReferenceKind;
        self.symbols.push(ResolvedSymbol {
            file: self.file.clone(),
            span,
            expr_span: None,
            kind: ReferenceKind::Variable(Arc::from(var_name)),
            resolved_type: ty,
        });
    }

    fn offset_to_line_col(&self, offset: u32) -> (u32, u16) {
        crate::diagnostics::offset_to_line_col(self.source, offset, self.source_map)
    }

    /// Convert a span to Location (line, line_end, col_start, col_end).
    fn span_to_location(&self, span: php_ast::Span) -> (u32, u32, u16, u16) {
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (line_end, col_end) = if span.start < span.end {
            self.offset_to_line_col(span.end)
        } else {
            (line, col_start)
        };
        (line, line_end, col_start, col_end)
    }

    /// Emit `UndefinedClass` for a `Name` AST node if the resolved class does not exist.
    fn check_name_undefined_class(&mut self, name: &php_ast::owned::Name) {
        let raw = crate::parser::name_to_string_owned(name);
        let resolved = crate::db::resolve_name(self.db, &self.file, &raw);
        if matches!(resolved.as_str(), "self" | "static" | "parent") {
            return;
        }
        if crate::db::class_exists(self.db, &resolved) {
            return;
        }
        let span = name.span;
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

    /// Extract suppression names from a parsed docblock string.
    fn extract_suppressions_from(&self, doc: Option<&str>) -> Vec<String> {
        let Some(doc) = doc else {
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

    fn extract_mir_checks_from(&self, doc: Option<&str>) -> Vec<(String, String)> {
        let Some(doc) = doc else {
            return vec![];
        };
        crate::parser::DocblockParser::parse(doc).mir_checks
    }

    /// Extract a `@var` annotation from a parsed docblock string.
    /// The type is resolved through the file's imports/namespace.
    fn extract_var_annotation_from(&self, doc: Option<&str>) -> Option<VarAnnotation> {
        let parsed = crate::parser::DocblockParser::parse(doc?);
        let ty = parsed.var_type?;
        Some(VarAnnotation {
            name: parsed.var_name,
            ty: resolve_union_for_file(ty, self.db, &self.file),
        })
    }

    // -----------------------------------------------------------------------
    // Fixed-point loop widening (M12)
    // -----------------------------------------------------------------------

    /// Analyse a loop body with a fixed-point widening algorithm (≤ 3 passes).
    ///
    /// * `pre`   — context *before* the loop (used as the merge base)
    /// * `entry` — context on first iteration entry (may be narrowed / seeded)
    /// * `body`  — closure that analyses one loop iteration, receives `&mut Self`
    ///   and `&mut FlowState` for the current iteration context
    /// * `loop_guaranteed` — whether the loop is guaranteed to execute at least once
    /// * `is_infinite` — whether the loop condition is always-true (while(true)/for(;;)),
    ///   meaning normal loop exit is unreachable and only break paths matter
    ///
    /// Returns the post-loop context that merges:
    ///   - the stable widened context after normal loop exit
    ///   - any contexts captured at `break` statements
    fn analyze_loop_widened<F>(
        &mut self,
        pre: &FlowState,
        entry: FlowState,
        mut body: F,
        loop_guaranteed: bool,
        is_infinite: bool,
    ) -> FlowState
    where
        F: FnMut(&mut Self, &mut FlowState),
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

            let mut next = FlowState::merge_branches(pre, iter.clone(), None);

            // When the loop body reads a variable that was pending before the loop,
            // the pre-loop write was consumed on the "loop ran" path.  The
            // merge_branches call above uses pre.clone() as the else-path ("loop
            // never ran"), which reintroduces those pre-loop pending writes into
            // the union.  Only remove a variable from the result when its current
            // location in `next` still matches the pre-loop location — meaning
            // the loop body read the old value but did NOT write a new one.
            // If the loop body wrote a new value (different location), keep it.
            for name in iter.read_vars.iter() {
                if let Some(&pre_loc) = pre.last_write_locs.get(name) {
                    if next.last_write_locs.get(name) == Some(&pre_loc) {
                        next.last_write_locs.remove(name);
                    }
                }
            }

            if vars_stabilized(&prev_vars, &next.vars) {
                current = next;
                break;
            }
            current = next;
        }

        // Widen any variable still unstable after MAX_ITERS to the union of types
        widen_unstable(
            &pre.vars,
            std::sync::Arc::make_mut(&mut current.vars),
            loop_guaranteed,
        );

        // For infinite loops (while(true)/for(;;)) the normal-exit path is unreachable;
        // only break statements can leave the loop. Marking current as diverging causes
        // merge_branches below to use the break context directly, so variables assigned
        // before every break are treated as definitely-assigned after the loop.
        if is_infinite {
            current.diverges = true;
        }

        // Pop break contexts and merge them into the post-loop result
        let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();
        for bctx in break_ctxs {
            current = FlowState::merge_branches(pre, current, Some(bctx));
        }

        current
    }
}

/// Widen literal types to their base scalar type for `@mir-check` comparisons.
/// `TLiteralInt(42)` → `TInt`, `TLiteralString("s")` → `TString`, etc.
pub(crate) fn widen_for_check(u: Type) -> Type {
    let mut out = Type::empty();
    for atomic in u.types {
        let widened = match atomic {
            Atomic::TLiteralInt(_) | Atomic::TIntRange { .. } => Atomic::TInt,
            Atomic::TLiteralString(_) => Atomic::TString,
            Atomic::TLiteralFloat(_, _) => Atomic::TFloat,
            other => other,
        };
        out.add_type(widened);
    }
    out
}
