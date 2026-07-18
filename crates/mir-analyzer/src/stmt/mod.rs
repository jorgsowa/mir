/// Statement analyzer — walks statement nodes threading context through
/// control flow (if/else, loops, try/catch, return).
mod control_flow;
mod declarations;
mod expressions;
mod flow;
mod loops;
mod return_type;

pub(crate) use loops::infer_foreach_types;
use loops::{vars_stabilized, widen_unstable};
pub(crate) use return_type::named_object_return_compatible;
pub(crate) use return_type::resolve_union_for_file;

use std::sync::Arc;

use crate::parser::docblock::parse_type_string;

use php_ast::owned::{Expr, ExprKind, StmtKind};

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

/// The `$name` (without `$`) of a simple `$x = expr;` statement LHS, if any.
fn simple_assignment_lhs(stmt: &php_ast::owned::Stmt) -> Option<&str> {
    let php_ast::owned::StmtKind::Expression(e) = &stmt.kind else {
        return None;
    };
    let php_ast::owned::ExprKind::Assign(a) = &e.kind else {
        return None;
    };
    if !matches!(&a.op, php_ast::ast::AssignOp::Assign) {
        return None;
    }
    let php_ast::owned::ExprKind::Variable(lhs_name) = &a.target.kind else {
        return None;
    };
    Some(lhs_name.trim_start_matches('$'))
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
    /// When false, `ResolvedSymbol` recording is skipped entirely. Set by
    /// pure inference walks whose caller discards the symbol buffer, so the
    /// walk doesn't clone a Type per reference just to drop it.
    pub collect_symbols: bool,
    /// Accumulated inferred return types for the current function.
    pub return_types: Vec<Type>,
    /// `(key type, value type)` for every `yield`/`yield from` seen so far in
    /// the current function body — see `ExpressionAnalyzer::yielded_types`.
    pub yielded_types: Vec<(Type, Type)>,
    /// Break-context stack: one entry per active loop/switch nesting level
    /// (both count towards break/continue level targeting in PHP).
    /// Each entry collects the context states at every `break`/switch-scoped
    /// `continue` targeting that level.
    break_ctx_stack: Vec<Vec<FlowState>>,
    /// Parallel to `break_ctx_stack`: `true` if the corresponding level is a
    /// real loop, `false` if it's a `switch`. `continue` behaves differently
    /// depending on which kind its target level is — see `analyze_continue_stmt`.
    loop_kind_stack: Vec<bool>,
    /// Snapshot of the installed plugin registry — see
    /// `ExpressionAnalyzer::plugins`.
    plugins: Option<std::sync::Arc<mir_plugin::PluginRegistry>>,
    /// Cache of the last `find_class_like` lookup done for `@var`-annotation
    /// alias expansion (`extract_var_annotation_from`), keyed by FQCN. Many
    /// consecutive `@var`-annotated statements within the same method/class
    /// body share the same `self_fqcn`; without this, each one repeats an
    /// `Arc<ClassDef>`-cloning workspace-index lookup that measurably added
    /// up across a large corpus (see the alias-expansion commit's perf note).
    class_like_cache: Option<(Arc<str>, Option<crate::db::ClassLike>)>,
    /// Cache of the last `find_function` lookup done for `@var`-annotation
    /// alias expansion in a free function's own body, mirroring
    /// `class_like_cache`.
    function_cache: Option<(Arc<str>, Option<Arc<mir_codebase::definitions::FunctionDef>>)>,
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
            collect_symbols: true,
            return_types: Vec::new(),
            yielded_types: Vec::new(),
            break_ctx_stack: Vec::new(),
            loop_kind_stack: Vec::new(),
            plugins: mir_plugin::snapshot(),
            class_like_cache: None,
            function_cache: None,
        }
    }

    /// `find_class_like(fqcn)`, memoized against the immediately preceding
    /// call — cheap when consecutive calls share the same FQCN (the common
    /// case: many `@var`-annotated statements in the same method body).
    fn cached_class_like(&mut self, fqcn: &str) -> Option<&crate::db::ClassLike> {
        let hit = self
            .class_like_cache
            .as_ref()
            .is_some_and(|(cached_fqcn, _)| cached_fqcn.as_ref() == fqcn);
        if !hit {
            let resolved =
                crate::db::find_class_like(self.db, crate::db::Fqcn::from_str(self.db, fqcn));
            self.class_like_cache = Some((Arc::from(fqcn), resolved));
        }
        self.class_like_cache
            .as_ref()
            .and_then(|(_, cl)| cl.as_ref())
    }

    /// `find_function(fqn)`, memoized against the immediately preceding call —
    /// mirrors `cached_class_like` for the free-function `@var` alias-expansion
    /// fallback below.
    fn cached_function(&mut self, fqn: &str) -> Option<Arc<mir_codebase::definitions::FunctionDef>> {
        let hit = self
            .function_cache
            .as_ref()
            .is_some_and(|(cached_fqn, _)| cached_fqn.as_ref() == fqn);
        if !hit {
            let resolved =
                crate::db::find_function(self.db, crate::db::Fqcn::from_str(self.db, fqn));
            self.function_cache = Some((Arc::from(fqn), resolved));
        }
        self.function_cache.as_ref().and_then(|(_, f)| f.clone())
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
                            col_end: crate::diagnostics::clamp_col_end(
                                line, line_end, col_start, col_end,
                            ),
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

        let var_annotation = self.extract_var_annotation_from(
            doc.as_deref(),
            ctx.self_fqcn.as_deref(),
            ctx.current_function_fqn.as_deref(),
        );

        // Pre-narrow: `@var Type $varname` before any statement narrows that variable.
        if let Some(ref ann) = var_annotation {
            // UndefinedDocblockClass: `@var` names a class that doesn't exist;
            // otherwise record it as a `cls:` reference so a class named only
            // via a local `@var` assertion isn't falsely flagged UnusedClass.
            for atomic in &ann.ty.types {
                if let Atomic::TNamedObject { fqcn, .. } = atomic {
                    if crate::diagnostics::is_pseudo_type(fqcn.as_ref()) {
                        continue;
                    }
                    let (line, line_end, col_start, col_end) = self.span_to_location(stmt.span);
                    if !crate::db::class_exists(self.db, fqcn.as_ref()) {
                        self.issues.add(Issue::new(
                            IssueKind::UndefinedDocblockClass {
                                name: fqcn.to_string(),
                            },
                            Location {
                                file: self.file.clone(),
                                line,
                                line_end,
                                col_start,
                                col_end: crate::diagnostics::clamp_col_end(
                                    line, line_end, col_start, col_end,
                                ),
                            },
                        ));
                    } else if self.mode == AnalysisMode::Full {
                        self.db.record_reference_location(crate::db::RefLoc {
                            symbol_key: Arc::from(format!("cls:{fqcn}")),
                            file: self.file.clone(),
                            line,
                            col_start,
                            col_end: crate::diagnostics::clamp_col_end(
                                line, line_end, col_start, col_end,
                            ),
                        });
                    }
                }
            }
            if let Some(ref name) = ann.name {
                let current = ctx.get_var(name.as_str());
                // `@var` is normally a legitimate widening/narrowing assertion
                // (e.g. `Animal` -> `Dog`) that must stay silent — only flag it
                // when the two types share NO possible overlap at all (e.g.
                // `string` vs `int`), using coarse PHP type families so this
                // never fires on object narrowing (both sides just map to the
                // same OBJECT family bit) or on mixed/template/unknown types
                // (family mask 0, treated as "nothing to compare").
                let current_mask = crate::body_analysis::type_family_mask(&current);
                let ann_mask = crate::body_analysis::type_family_mask(&ann.ty);
                if current_mask != 0 && ann_mask != 0 && current_mask & ann_mask == 0 {
                    let (line, line_end, col_start, col_end) = self.span_to_location(stmt.span);
                    self.issues.add(Issue::new(
                        IssueKind::DocblockTypeContradiction {
                            expr: format!("@var {} ${name}", ann.ty),
                            declared: current.to_string(),
                        },
                        Location {
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
                ctx.set_var(name.as_str(), ann.ty.clone());
            }
        }

        // Check `@mir-check` directives
        let mir_checks = self.extract_mir_checks_from(doc.as_deref());
        let (line, line_end, col_start, col_end) = self.span_to_location(stmt.span);
        for (expr_text, expected_str) in mir_checks {
            let expected_raw = parse_type_string(&expected_str);
            // Resolve the expected type through the file's namespace/imports so that
            // `\Foo\Bar` in a @mir-check directive matches the stored `Foo\Bar` (sans `\`).
            let expected = resolve_union_for_file(expected_raw, self.db, &self.file);
            let actual_raw = self.expr_analyzer(ctx).eval_check_expr(&expr_text, ctx);
            if !mir_check_matches(&expected, &actual_raw) {
                self.issues.add(Issue::new(
                    IssueKind::TypeCheckMismatch {
                        var: expr_text,
                        expected: expected.to_string(),
                        actual: widen_for_check(actual_raw).to_string(),
                    },
                    Location {
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

        // Emit `@trace $var` directives
        if let Some(doc_str) = doc.as_deref() {
            let trace_vars = crate::parser::DocblockParser::parse(doc_str).trace_vars;
            for var_name in trace_vars {
                let ty = ctx.get_var(&var_name);
                self.issues.add(Issue::new(
                    IssueKind::Trace {
                        variable: var_name,
                        type_info: ty.to_string(),
                    },
                    Location {
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
            StmtKind::Break(level) => {
                self.analyze_break_stmt(ctx, break_continue_level(level));
            }

            // ---- Continue ----------------------------------------------------
            StmtKind::Continue(level) => {
                self.analyze_continue_stmt(ctx, break_continue_level(level));
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

            // ---- Namespace / use (at file level, already handled in definition
            // collection; braced namespace bodies are walked by
            // `BodyAnalyzer::analyze_global_exec` / `analyze_top_level_stmts`) --
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
            // An annotation that exactly matches the inferred (widened) type
            // of a simple assignment adds nothing — UnnecessaryVarAnnotation.
            // Narrowing or widening annotations stay silent.
            if self.mode == AnalysisMode::Full {
                if let Some(lhs) = simple_assignment_lhs(stmt) {
                    let applies = ann.name.as_deref().is_none_or(|n| n == lhs);
                    if applies {
                        let ann_ty = crate::expr::helpers::resolve_named_objects_in_union(
                            ann.ty.clone(),
                            self.db,
                            self.file.as_ref(),
                        );
                        // Exact comparison, NO literal widening: `@var string`
                        // on `$s = 'hello'` deliberately widens the literal
                        // (it changes conditional-return resolution), so it is
                        // not unnecessary.
                        let inferred = ctx.get_var(lhs);
                        if !inferred.is_mixed() && inferred.to_string() == ann_ty.to_string() {
                            self.issues.add(Issue::new(
                                IssueKind::UnnecessaryVarAnnotation {
                                    var: lhs.to_string(),
                                },
                                Location {
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
            apply_post_narrow(stmt, ann, ctx);
        }

        if let Some(plugins) = self.plugins.clone() {
            if plugins.hooks().after_statement_analysis {
                let file = self.file.clone();
                let mut event = mir_plugin::AfterStatementAnalysisEvent {
                    stmt,
                    file: file.as_ref(),
                    issues: Vec::new(),
                };
                plugins.after_statement_analysis(&mut event);
                let issues = event.issues;
                self.emit_plugin_issues(issues, stmt.span);
            }
        }

        // Runs after the plugin hook so statement-level `@mir-suppress` also
        // covers plugin-raised issues.
        if !suppressions.is_empty() {
            self.issues.suppress_range(before, &suppressions);
        }
    }

    /// Convert plugin-raised issues into diagnostics — statement-level twin
    /// of `ExpressionAnalyzer::emit_plugin_issues`.
    fn emit_plugin_issues(
        &mut self,
        issues: Vec<mir_plugin::PluginIssue>,
        default_span: php_ast::Span,
    ) {
        for pi in issues {
            let span = pi.span.unwrap_or(default_span);
            let (line, line_end, col_start, col_end) = self.span_to_location(span);
            let mut issue = Issue::new(
                IssueKind::PluginIssue {
                    name: pi.name,
                    message: pi.message,
                },
                Location {
                    file: self.file.clone(),
                    line,
                    line_end,
                    col_start,
                    col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
                },
            );
            issue.severity = pi.severity;
            if let Some(text) = crate::parser::span_text(self.source, span) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    issue.snippet = Some(trimmed.to_string());
                }
            }
            self.issues.add(issue);
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
            &mut self.yielded_types,
        );
        ea.strict_types = ctx.strict_types;
        ea.collect_symbols = self.collect_symbols;
        ea
    }

    fn record_symbol_for_var(&mut self, span: php_ast::Span, var_name: &str, ty: Type) {
        use crate::symbol::ReferenceKind;
        if !self.collect_symbols {
            return;
        }
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
                col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
            },
        ));
    }

    /// Record `resolved` as referenced, if it names a real class/interface/
    /// trait/enum. Used for an anonymous class's `extends`/`implements`/
    /// `use` targets, which — unlike a top-level class (checked via
    /// `check_name_class`/`check_trait_constraints`, both of which record
    /// refs) — are never collected into the codebase, so nothing else ever
    /// marks these names as used. Without this, a class/trait reachable only
    /// through an anonymous class's clause is falsely treated as unreferenced.
    pub(crate) fn record_class_like_ref(&mut self, resolved: &str, span: php_ast::Span) {
        if self.mode != AnalysisMode::Full || !crate::db::class_exists(self.db, resolved) {
            return;
        }
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (line_end, col_end) = self.offset_to_line_col(span.end);
        self.db.record_reference_location(crate::db::RefLoc {
            symbol_key: Arc::from(format!("cls:{resolved}")),
            file: self.file.clone(),
            line,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        });
    }

    /// Emit `UndefinedTrait` for a `use` clause's trait name if it does not
    /// resolve to a real trait. Used by anonymous (and nested named) class
    /// declarations, which — unlike a top-level class — are never collected
    /// into the codebase, so `check_trait_constraints`'s definition-based
    /// walk never sees them.
    pub(crate) fn check_name_undefined_trait(&mut self, name: &php_ast::owned::Name) {
        let raw = crate::parser::name_to_string_owned(name);
        let resolved = crate::db::resolve_name(self.db, &self.file, &raw);
        if crate::db::class_exists(self.db, &resolved) {
            return;
        }
        let short = resolved
            .rsplit('\\')
            .next()
            .unwrap_or(resolved.as_str())
            .to_string();
        let span = name.span;
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (line_end, col_end) = self.offset_to_line_col(span.end);
        self.issues.add(Issue::new(
            IssueKind::UndefinedTrait { name: short },
            Location {
                file: self.file.clone(),
                line,
                line_end,
                col_start,
                col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
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
    /// `self_fqcn` — the class/interface/trait/enum whose body is currently
    /// being analysed (`FlowState::self_fqcn`), if any — supplies that
    /// class-like's own `@psalm-type`/`@phpstan-type` aliases so a bare
    /// `@var Result $x` expands exactly like a `@param`/`@return` reference
    /// to the same alias already does (see `collector::build_method_storage`
    /// / the "expand aliases first, then resolve" precedent). Expansion runs
    /// before `resolve_union_for_file` so the `UndefinedDocblockClass`/
    /// reference-recording checks in `analyze_stmt` see the real referenced
    /// class(es), not the opaque alias name. When `self_fqcn` is `None` (a
    /// free function's body, or a nested function decl that doesn't set
    /// `current_function_fqn`), `current_function_fqn` supplies that
    /// function's own `@psalm-type`/`@phpstan-type` aliases the same way.
    fn extract_var_annotation_from(
        &mut self,
        doc: Option<&str>,
        self_fqcn: Option<&str>,
        current_function_fqn: Option<&str>,
    ) -> Option<VarAnnotation> {
        let parsed = crate::parser::DocblockParser::parse(doc?);
        let mut ty = parsed.var_type?;
        if let Some(fqcn) = self_fqcn {
            if let Some(class_like) = self.cached_class_like(fqcn) {
                let aliases = class_like.type_aliases();
                if !aliases.is_empty() {
                    ty = crate::collector::expand_aliases_only(ty, aliases);
                }
            }
        } else if let Some(fqn) = current_function_fqn {
            if let Some(function) = self.cached_function(fqn) {
                if !function.type_aliases.is_empty() {
                    ty = crate::collector::expand_aliases_only(ty, &function.type_aliases);
                }
            }
        }
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
        exit_condition: Option<&Expr>,
    ) -> FlowState
    where
        F: FnMut(&mut Self, &mut FlowState),
    {
        const MAX_ITERS: usize = 3;

        // Push a fresh break-context bucket for this loop level
        self.break_ctx_stack.push(Vec::new());
        self.loop_kind_stack.push(true);

        let mut current = entry;
        current.inside_loop = true;

        for iter_idx in 0..MAX_ITERS {
            let prev_vars = current.vars.clone();

            // `body` runs the real statement analyzer, which emits diagnostics
            // directly into the shared issue buffer — not a scratch copy. Only
            // the FINAL, stabilized pass reflects a realistic "any iteration"
            // type state; earlier passes see overly-narrow entry
            // approximations (e.g. a loop counter still at 0 or 1, a toggled
            // bool still `true`) and can flag things that are only true on
            // that unstabilized pass — an `if ($first)` toggle read as always
            // true, or a comparison against a value the loop variable hasn't
            // widened to yet read as always false. Mark the buffer here and
            // roll back below if this pass turns out not to be the last one.
            let issues_mark = self.issues.issue_count();

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
                if let Some(pre_locs) = pre.last_write_locs.get(name) {
                    if let Some(locs) = next.last_write_locs.get_mut(name) {
                        locs.retain(|l| !pre_locs.contains(l));
                        if locs.is_empty() {
                            next.last_write_locs.remove(name);
                        }
                    }
                }
            }

            if vars_stabilized(&prev_vars, &next.vars) {
                current = next;
                break;
            }
            // Not the fixed point yet, and (since the loop keeps going) not
            // the last pass either — this pass's diagnostics were computed
            // against a type state later passes will refine, so they're
            // provisional. Roll them back; a real issue at the same
            // condition will re-emit on a later pass if it's still real.
            if iter_idx + 1 < MAX_ITERS {
                self.issues.truncate_to(issues_mark);
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
        } else if let Some(cond) = exit_condition {
            // A non-break exit from a finite loop can only happen when the loop's
            // own condition evaluates false — apply that negated narrowing to
            // `current` (the "ran zero-or-more times, then condition failed" path)
            // BEFORE merging in the break contexts below, since a `break` exits
            // while the condition was still true (or mid-body) and must not be
            // narrowed by it.
            crate::narrowing::narrow_from_condition(cond, &mut current, false, self.db, &self.file);
        }

        // Pop break contexts and merge them into the post-loop result
        let break_ctxs = self.break_ctx_stack.pop().unwrap_or_default();
        self.loop_kind_stack.pop();
        for bctx in break_ctxs {
            current = FlowState::merge_branches(pre, current, Some(bctx));
        }

        current
    }
}

/// The loop-nesting level a `break`/`continue` statement targets. PHP only
/// accepts an integer-literal argument here (a non-literal is a compile
/// error), so any other shape (including `None`, i.e. bare `break;`) is
/// treated as the default level 1.
fn break_continue_level(level_expr: &Option<Box<Expr>>) -> usize {
    match level_expr.as_deref().map(|e| &e.kind) {
        Some(ExprKind::Int(n)) if *n >= 1 => *n as usize,
        _ => 1,
    }
}

/// Whether an `@mir-check $x is T` directive is satisfied by the inferred type.
///
/// The directive matches if the inferred type equals the expected type either
/// *exactly* or after [`widen_for_check`] widening. The exact arm lets fixtures
/// assert precise types the analyzer produces — integer ranges (`int<0, max>`),
/// literals — while the widened arm keeps the lenient default: writing
/// `@mir-check $x is int` still passes when `$x` infers to `int<0, max>` or a
/// literal. Widening is a one-directional relaxation, so this is strictly more
/// permissive than an exact-only check and never breaks an existing fixture.
pub(crate) fn mir_check_matches(expected: &Type, actual: &Type) -> bool {
    let expected_str = expected.to_string();
    expected_str == actual.to_string()
        || expected_str == widen_for_check(actual.clone()).to_string()
}

/// Widen literal types to their base scalar type. `TLiteralInt(42)` → `TInt`,
/// `TLiteralString("s")` → `TString`, etc. Used both for `@mir-check`
/// comparisons and to avoid carrying an over-narrow literal into a
/// substituted template binding.
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
