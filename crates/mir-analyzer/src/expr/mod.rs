/// Expression analyzer — infers the `Type` type of any PHP expression.
use std::sync::Arc;

use php_ast::owned::ExprKind;

use mir_issues::{Issue, IssueBuffer, IssueKind, Location, Severity};
use mir_types::{Atomic, CloneValidity, Type};

use crate::body_analysis::AnalysisMode;
use crate::db::MirDatabase;
use crate::flow_state::FlowState;
use crate::php_version::PhpVersion;
use crate::symbol::{ReferenceKind, ResolvedSymbol};

mod arrays;
mod assignment;
mod binary;
mod casts;
mod closures;
mod conditional;
pub(crate) mod helpers;
mod intrinsics;
mod literals;
mod objects;
mod unary;
mod variables;

pub(crate) use binary::operand_is_definitely_zero;
#[allow(unused_imports)]
pub use helpers::{duplicate_literal_conditions, extract_simple_var, infer_arithmetic, infer_div};

/// Parses `text` as a standalone PHP expression by wrapping it as a
/// throwaway one-statement program — the parser crate only exposes a
/// full-program entry point, no snippet-only expression parser. Returns
/// `None` on any parse error or if the snippet isn't a single expression
/// statement.
fn parse_check_expr(text: &str) -> Option<php_ast::owned::Expr> {
    let wrapped = format!("<?php {text};");
    let result = php_rs_parser::parse(&wrapped);
    if !result.errors.is_empty() {
        return None;
    }
    let mut stmts = result.program.stmts.into_vec();
    if stmts.len() != 1 {
        return None;
    }
    match stmts.remove(0).kind {
        php_ast::owned::StmtKind::Expression(expr) => Some(*expr),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// ExpressionAnalyzer
// ---------------------------------------------------------------------------

pub struct ExpressionAnalyzer<'a> {
    pub db: &'a dyn MirDatabase,
    pub file: Arc<str>,
    pub source: &'a str,
    pub source_map: &'a php_rs_parser::source_map::SourceMap,
    pub issues: &'a mut IssueBuffer,
    pub symbols: &'a mut Vec<ResolvedSymbol>,
    pub php_version: PhpVersion,
    pub mode: AnalysisMode,
    /// Whether `declare(strict_types=1)` is active for the calling file.
    /// When true, coercive PHP typing (e.g. Stringable → string) must not be
    /// silently allowed — the runtime would throw a TypeError.
    pub strict_types: bool,
    /// When false, `record_symbol*` calls are no-ops — see
    /// `StatementsAnalyzer::collect_symbols`.
    pub collect_symbols: bool,
    /// When true, we are inside an existence-check context (isset/empty/??) where missing
    /// variables and missing array offsets are not errors — they are what is being tested.
    in_existence_check: bool,
    /// `(key type, value type)` for every `yield`/`yield from` encountered so
    /// far in the enclosing function body, regardless of which branch/loop it
    /// syntactically sits in — the AST is walked once, so no branch-merge
    /// logic is needed. Borrowed from the owning `StatementsAnalyzer` so it
    /// survives across the many short-lived `ExpressionAnalyzer`s created
    /// while walking one function body. Read back by `body_analysis` to
    /// build the function's inferred `Generator<K, V, S, R>` return type.
    pub(crate) yielded_types: &'a mut Vec<(Type, Type)>,
    /// Snapshot of the installed plugin registry (`None` when no plugins are
    /// loaded — the common case, making every hook site a single check).
    pub(crate) plugins: Option<std::sync::Arc<mir_plugin::PluginRegistry>>,
}

impl<'a> ExpressionAnalyzer<'a> {
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
        yielded_types: &'a mut Vec<(Type, Type)>,
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
            strict_types: false,
            collect_symbols: true,
            in_existence_check: false,
            yielded_types,
            plugins: mir_plugin::snapshot(),
        }
    }

    /// Run `f` in an existence-check context (isset/empty/??/??=), suppressing
    /// missing-variable and missing-offset diagnostics for the duration.
    pub(super) fn with_existence_check<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let old = self.in_existence_check;
        self.in_existence_check = true;
        let result = f(self);
        self.in_existence_check = old;
        result
    }

    /// Evaluate a test-only expression snippet (from an `@mir-check`/`@trace`
    /// docblock directive, e.g. `$h->status` or `self::$prop`) against a flow
    /// state, reusing the same `analyze` every other expression goes through
    /// so any shape mir can already reason about is checkable — property
    /// access, static properties, array/shape keys, and anything narrowing
    /// later learns to handle, with no per-kind plumbing to maintain here.
    /// Runs against a cloned `FlowState` and discards any issues/symbols the
    /// check itself would have produced: it's an assertion about existing
    /// state, not new analysis, so it must not leak into the real diagnostic
    /// set. Returns `Type::mixed()` if `expr_text` fails to parse.
    pub fn eval_check_expr(&mut self, expr_text: &str, ctx: &FlowState) -> Type {
        let Some(expr) = parse_check_expr(expr_text) else {
            return Type::mixed();
        };
        let mark = self.issues.issue_count();
        let had_collect_symbols = self.collect_symbols;
        self.collect_symbols = false;
        let mut scratch_ctx = ctx.clone();
        let ty = self.analyze(&expr, &mut scratch_ctx);
        self.collect_symbols = had_collect_symbols;
        self.issues.truncate_to(mark);
        ty
    }

    /// Record a resolved symbol.
    pub fn record_symbol(&mut self, span: php_ast::Span, kind: ReferenceKind, resolved_type: Type) {
        if !self.collect_symbols {
            return;
        }
        self.symbols.push(ResolvedSymbol {
            file: self.file.clone(),
            span,
            expr_span: None,
            kind,
            resolved_type,
        });
    }

    pub fn record_symbol_with_expr_span(
        &mut self,
        span: php_ast::Span,
        expr_span: php_ast::Span,
        kind: ReferenceKind,
        resolved_type: Type,
    ) {
        if !self.collect_symbols {
            return;
        }
        self.symbols.push(ResolvedSymbol {
            file: self.file.clone(),
            span,
            expr_span: Some(expr_span),
            kind,
            resolved_type,
        });
    }

    /// Record a member-access receiver's type at the gap between
    /// `receiver_span` and `member_span` — the `->`/`?->`/`::` operator (and
    /// any whitespace around it). `symbol_at`'s primary lookup matches the
    /// smallest recorded span containing the query offset, so this only ever
    /// wins at offsets no more precise symbol already covers (e.g. inside the
    /// operator itself) — it does not shadow the member's own resolved-type
    /// symbol or a call's `expr_span` fallback.
    pub fn record_receiver_type(
        &mut self,
        receiver_span: php_ast::Span,
        member_span: php_ast::Span,
        ty: Type,
    ) {
        if receiver_span.end >= member_span.start {
            // No gap to record into — adjacent/overlapping/malformed spans.
            return;
        }
        self.record_symbol(
            php_ast::Span::new(receiver_span.end, member_span.start),
            ReferenceKind::Receiver,
            ty,
        );
    }

    pub fn analyze(&mut self, expr: &php_ast::owned::Expr, ctx: &mut FlowState) -> Type {
        let ty = self.analyze_inner(expr, ctx);
        if let Some(plugins) = self.plugins.clone() {
            if plugins.hooks().after_expression_analysis {
                let file = self.file.clone();
                let mut event = mir_plugin::AfterExpressionAnalysisEvent {
                    expr,
                    expr_type: &ty,
                    file: file.as_ref(),
                    issues: Vec::new(),
                };
                plugins.after_expression_analysis(&mut event);
                let issues = event.issues;
                self.emit_plugin_issues(issues, expr.span);
            }
        }
        ty
    }

    fn analyze_inner(&mut self, expr: &php_ast::owned::Expr, ctx: &mut FlowState) -> Type {
        match &expr.kind {
            // --- Literals ---------------------------------------------------
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Bool(_)
            | ExprKind::Null => literals::analyze(&expr.kind),

            ExprKind::InterpolatedString(parts) | ExprKind::Heredoc { parts, .. } => {
                // A heredoc (or, in principle, a double-quoted string) with no
                // embedded expression parts is just as much a compile-time
                // literal as an equivalent quoted string — fold it to
                // TLiteralString so callable-string usage tracking, class-string
                // reflection, narrowing, and match/switch dedup (all of which key
                // off TLiteralString) don't silently stop working just because
                // the literal happens to be spelled as a heredoc.
                let mut literal = String::new();
                let mut all_literal = true;
                for part in parts.iter() {
                    match part {
                        php_ast::owned::StringPart::Literal(s) => literal.push_str(s),
                        php_ast::owned::StringPart::Expr(e) => {
                            all_literal = false;
                            let expr_ty = self.analyze(e, ctx);
                            self.check_interpolation_implicit_to_string_cast(&expr_ty, e.span);
                        }
                    }
                }
                if all_literal {
                    Type::single(Atomic::TLiteralString(literal.into()))
                } else {
                    Type::single(Atomic::TString)
                }
            }
            ExprKind::Nowdoc { value, .. } => {
                Type::single(Atomic::TLiteralString(value.as_ref().into()))
            }
            ExprKind::ShellExec(_) => {
                self.emit(
                    IssueKind::ForbiddenCode {
                        message: "Use of shell_exec (backtick) is forbidden".to_string(),
                    },
                    Severity::Warning,
                    expr.span,
                );
                Type::single(Atomic::TString)
            }

            // --- Variables --------------------------------------------------
            ExprKind::Variable(name) => self.analyze_variable(name.as_ref(), expr, ctx),
            ExprKind::VariableVariable(inner) => self.analyze_variable_variable(inner, ctx),
            ExprKind::Identifier(name) => self.analyze_identifier(name.as_ref(), expr, ctx),

            // --- Assignment -------------------------------------------------
            ExprKind::Assign(a) => self.analyze_assign(a, expr.span, ctx),

            // --- Binary operations ------------------------------------------
            ExprKind::Binary(b) => self.analyze_binary_expr(b, expr.span, ctx),

            // --- Unary ------------------------------------------------------
            ExprKind::UnaryPrefix(u) => self.analyze_unary_prefix(u, ctx),
            ExprKind::UnaryPostfix(u) => self.analyze_unary_postfix(u, ctx),

            // --- Ternary / null coalesce ------------------------------------
            ExprKind::Ternary(t) => self.analyze_ternary(t, ctx),
            ExprKind::NullCoalesce(nc) => self.analyze_null_coalesce(nc, ctx),

            // --- Casts ------------------------------------------------------
            ExprKind::Cast(kind, inner) => self.analyze_cast(kind, inner, ctx),

            // --- Error suppression ------------------------------------------
            ExprKind::ErrorSuppress(inner) => self.analyze(inner, ctx),

            // --- Parenthesized ----------------------------------------------
            ExprKind::Parenthesized(inner) => self.analyze(inner, ctx),

            // --- Array literals ---------------------------------------------
            ExprKind::Array(elements) => self.analyze_array(elements, ctx),

            // --- Array access -----------------------------------------------
            ExprKind::ArrayAccess(aa) => self.analyze_array_access(aa, expr, ctx),

            // --- isset / empty ----------------------------------------------
            ExprKind::Isset(exprs) => {
                self.with_existence_check(|ea| {
                    for e in exprs.iter() {
                        ea.analyze(e, ctx);
                    }
                });
                Type::single(Atomic::TBool)
            }
            ExprKind::Empty(inner) => {
                self.with_existence_check(|ea| ea.analyze(inner, ctx));
                Type::single(Atomic::TBool)
            }

            // --- print ------------------------------------------------------
            ExprKind::Print(inner) => {
                let expr_ty = self.analyze(inner, ctx);
                self.check_interpolation_implicit_to_string_cast(&expr_ty, inner.span);
                // `print` is an HTML sink exactly like `echo` — both write
                // straight to the response body.
                if crate::taint::is_expr_tainted(inner, ctx) {
                    self.emit(IssueKind::TaintedHtml, Severity::Error, expr.span);
                }
                // @pure implies no side effects at all — same purity
                // violation `echo` now checks (stmt/expressions.rs), for the
                // `print` expression form.
                if ctx.is_in_pure_fn {
                    self.emit(
                        IssueKind::ImpureFunctionCall {
                            fn_name: "print".to_string(),
                        },
                        Severity::Warning,
                        expr.span,
                    );
                }
                Type::single(Atomic::TLiteralInt(1))
            }

            // --- clone ------------------------------------------------------
            ExprKind::Clone(inner) => {
                let ty = self.analyze(inner, ctx);
                self.check_clone_target(&ty, expr.span);
                self.check_clone_visibility(&ty, expr.span, ctx.self_fqcn.as_deref());
                self.check_clone_deprecated(&ty, expr.span);
                ty
            }
            ExprKind::CloneWith(inner, _props) => {
                let ty = self.analyze(inner, ctx);
                self.check_clone_target(&ty, expr.span);
                self.check_clone_visibility(&ty, expr.span, ctx.self_fqcn.as_deref());
                ty
            }

            // --- new ClassName(...) ----------------------------------------
            ExprKind::New(n) => self.analyze_new(n, expr.span, ctx),

            // --- Anonymous class -------------------------------------------
            ExprKind::AnonymousClass(anon) => {
                // Record subtype markers: anonymous classes never reach the
                // definition collector, so `impl:{parent}` postings are how
                // goto-implementation finds `new class implements X {}`.
                let kw_start = self
                    .source
                    .get(expr.span.start as usize..)
                    .and_then(|tail| tail.find("class"))
                    .map(|p| expr.span.start + p as u32)
                    .unwrap_or(expr.span.start);
                let kw_span = php_ast::Span {
                    start: kw_start,
                    end: kw_start + "class".len() as u32,
                };
                let mut anon_supers: Vec<String> = Vec::new();
                if let Some(parent) = &anon.extends {
                    anon_supers.push(crate::parser::name_to_string_owned(parent));
                }
                for iface in anon.implements.iter() {
                    anon_supers.push(crate::parser::name_to_string_owned(iface));
                }
                for name in anon_supers {
                    let resolved = crate::db::resolve_name(self.db, self.file.as_ref(), &name);
                    let lc = resolved.trim_start_matches('\\').to_ascii_lowercase();
                    self.record_ref(Arc::from(format!("impl:{lc}")), kw_span);
                    let short = lc.rsplit('\\').next().unwrap_or(&lc);
                    if short != lc {
                        self.record_ref(Arc::from(format!("implshort:{short}")), kw_span);
                    } else {
                        self.record_ref(Arc::from(format!("implshort:{lc}")), kw_span);
                    }
                }
                let mut sa = crate::stmt::StatementsAnalyzer::new(
                    self.db,
                    self.file.clone(),
                    self.source,
                    self.source_map,
                    self.issues,
                    self.symbols,
                    self.php_version,
                    self.mode,
                );
                sa.collect_symbols = self.collect_symbols;
                sa.analyze_class_decl_stmt(anon, ctx);
                Type::single(Atomic::TObject)
            }

            // --- Property access -------------------------------------------
            ExprKind::PropertyAccess(pa) => self.analyze_property_access(pa, expr.span, ctx),

            ExprKind::NullsafePropertyAccess(pa) => self.analyze_nullsafe_property_access(pa, ctx),

            ExprKind::StaticPropertyAccess(spa) => {
                if matches!(&spa.class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(&spa.class, ctx);
                }
                self.analyze_static_property_access(spa, ctx)
            }

            ExprKind::ClassConstAccess(cca) => {
                // When the class part is a variable (e.g. `$obj::class`, `$obj::CONST`),
                // analyze it so the variable read is tracked and the write is consumed.
                if matches!(&cca.class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(&cca.class, ctx);
                }
                self.analyze_class_const_access(cca, expr.span, ctx)
            }

            ExprKind::ClassConstAccessDynamic { class, member } => {
                if matches!(&class.kind, ExprKind::Variable(_)) {
                    let _ = self.analyze(class, ctx);
                }
                let _ = self.analyze(member, ctx);
                Type::mixed()
            }
            ExprKind::StaticPropertyAccessDynamic { class, member } => {
                if let ExprKind::Identifier(name) = &class.kind {
                    let resolved = crate::db::resolve_name(self.db, self.file.as_ref(), name);
                    let fqcn = match ctx.self_fqcn.as_deref() {
                        Some(self_fqcn) if resolved.eq_ignore_ascii_case("self") => {
                            self_fqcn.to_string()
                        }
                        _ => resolved,
                    };
                    if !matches!(fqcn.as_str(), "self" | "static" | "parent") {
                        self.record_ref(Arc::from(format!("dyn:{fqcn}")), member.span);
                    }
                } else {
                    let class_ty = self.analyze(class, ctx);
                    self.record_dynamic_member_access(&class_ty, member.span);
                }
                let _ = self.analyze(member, ctx);
                Type::mixed()
            }

            // --- Method calls ----------------------------------------------
            ExprKind::MethodCall(mc) => {
                crate::call::CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, false)
            }

            ExprKind::NullsafeMethodCall(mc) => {
                crate::call::CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, true)
            }

            ExprKind::StaticMethodCall(smc) => {
                crate::call::CallAnalyzer::analyze_static_method_call(self, smc, ctx, expr.span)
            }

            ExprKind::StaticDynMethodCall(smc) => {
                crate::call::CallAnalyzer::analyze_static_dyn_method_call(self, smc, ctx)
            }

            // --- Function calls --------------------------------------------
            ExprKind::FunctionCall(fc) => {
                crate::call::CallAnalyzer::analyze_function_call(self, fc, ctx, expr.span)
            }

            // --- Closures / arrow functions --------------------------------
            ExprKind::Closure(c) => self.analyze_closure(c, expr.span, ctx),

            ExprKind::ArrowFunction(af) => self.analyze_arrow_function(af, expr.span, ctx),

            ExprKind::CallableCreate(cc) => self.callable_create_type(cc, expr.span, ctx),

            // --- Match expression ------------------------------------------
            ExprKind::Match(m) => self.analyze_match(m, expr.span, ctx),

            // --- Throw as expression (PHP 8) --------------------------------
            ExprKind::ThrowExpr(e) => {
                self.analyze(e, ctx);
                Type::single(Atomic::TNever)
            }

            // --- Yield -----------------------------------------------------
            ExprKind::Yield(y) => self.analyze_yield(y, ctx),

            // --- Magic constants -------------------------------------------
            ExprKind::MagicConst(kind) => ExpressionAnalyzer::analyze_magic_const(kind),

            // --- Include/require --------------------------------------------
            ExprKind::Include(_, inner) => {
                self.analyze(inner, ctx);
                // A require/include can read any variable currently in scope, so
                // mark all pending writes as consumed and mark the names as read.
                // This covers both the last_write_locs path and the assigned_vars
                // fallback in emit_unused_variables.
                let names: Vec<mir_types::Name> = ctx.last_write_locs.keys().copied().collect();
                for name in names {
                    ctx.read_vars.insert(name);
                }
                for (name, locs) in ctx.last_write_locs.drain() {
                    for loc in locs {
                        ctx.consumed_write_locs.insert((name, loc));
                    }
                }
                Type::mixed()
            }

            // --- Eval -------------------------------------------------------
            ExprKind::Eval(inner) => {
                self.analyze(inner, ctx);
                Type::mixed()
            }

            // --- Exit -------------------------------------------------------
            ExprKind::Exit(opt) => {
                if let Some(e) = opt {
                    self.analyze(e, ctx);
                }
                ctx.diverges = true;
                Type::single(Atomic::TNever)
            }

            // --- Error node (parse error placeholder) ----------------------
            ExprKind::Error => Type::mixed(),

            // --- Omitted array slot (e.g. [, $b] destructuring) ------------
            ExprKind::Omit => Type::single(Atomic::TNull),
        }
    }

    // -----------------------------------------------------------------------
    // Issue emission
    // -----------------------------------------------------------------------

    fn offset_to_line_col(&self, offset: u32) -> (u32, u16) {
        crate::diagnostics::offset_to_line_col(self.source, offset, self.source_map)
    }

    /// Convert an AST span to `(line, col_start, col_end)` for reference recording.
    fn callable_create_type(
        &mut self,
        cc: &php_ast::owned::CallableCreateExpr,
        outer_span: php_ast::Span,
        ctx: &mut FlowState,
    ) -> Type {
        use php_ast::owned::CallableCreateKind;
        match &cc.kind {
            CallableCreateKind::Function(name_expr) => {
                if let ExprKind::Identifier(name) = &name_expr.kind {
                    let resolved_fqn =
                        crate::db::resolve_name(self.db, self.file.as_ref(), name.as_ref());
                    let db = self.db;
                    let here = crate::db::Fqcn::from_str(db, &resolved_fqn);
                    if let Some(f) = crate::db::find_function(db, here) {
                        self.record_ref(Arc::from(format!("fn:{}", f.fqn)), name_expr.span);
                        if let Some((used, canonical)) =
                            crate::fqcn_case_mismatch(&resolved_fqn, f.fqn.as_ref())
                        {
                            let span = if name_expr.span.start < name_expr.span.end {
                                name_expr.span
                            } else {
                                outer_span
                            };
                            self.emit(
                                mir_issues::IssueKind::WrongCaseFunction { used, canonical },
                                mir_issues::Severity::Info,
                                span,
                            );
                        }
                        let return_ty = f
                            .return_type
                            .as_deref()
                            .cloned()
                            .unwrap_or_else(Type::mixed);
                        // Record a hover/go-to-definition symbol, matching the plain
                        // `foo()` call form — otherwise the direct-call path records
                        // both record_ref and record_symbol, but this first-class-
                        // callable form (`foo(...)`) only got the former, so
                        // find-references/dead-code worked but hover on the name
                        // token inside `(...)` silently resolved nothing.
                        self.record_symbol(
                            name_expr.span,
                            ReferenceKind::FunctionCall(f.fqn.clone()),
                            return_ty.clone(),
                        );
                        // No receiver for a plain function — self/static/parent can't
                        // legally appear in its signature, so an empty fqcn/type-param
                        // list is a safe no-op for build_closure_from_resolved_params's
                        // static-substitution step. Routed through the same helper as
                        // the method FCC cases below so the function's own @template
                        // params get bound to their declared bound (see the helper's
                        // doc comment) instead of leaking as bare, unchecked
                        // TTemplateParam atoms into every call through the closure.
                        let empty_fqcn: Arc<str> = Arc::from("");
                        return Type::single(Self::build_closure_from_resolved_params(
                            &f.params,
                            return_ty,
                            &rustc_hash::FxHashMap::default(),
                            &f.template_params,
                            &empty_fqcn,
                            &[],
                        ));
                    }
                }
                Type::single(Atomic::TCallable {
                    params: None,
                    return_type: None,
                })
            }

            CallableCreateKind::Method { object, method }
            | CallableCreateKind::NullsafeMethod { object, method } => {
                let nullsafe = matches!(&cc.kind, CallableCreateKind::NullsafeMethod { .. });
                let obj_ty = self.analyze(object, ctx);
                let method_name = match &method.kind {
                    ExprKind::Identifier(name) => name.clone(),
                    _ => {
                        self.analyze(method, ctx);
                        // Method name isn't statically known — same coarse
                        // exemption the ordinary `$obj->$name(...)` dynamic call
                        // gets, or a private method reachable only via
                        // `$obj->$name(...)` (first-class-callable form) is
                        // falsely flagged unused.
                        self.record_dynamic_member_access(&obj_ty, method.span);
                        return Type::single(Atomic::TCallable {
                            params: None,
                            return_type: None,
                        });
                    }
                };
                let method_name_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                if let Some((fqcn, receiver_type_params)) =
                    obj_ty.remove_null().types.iter().find_map(|a| {
                        a.named_object_fqcn().map(|fqcn| {
                            let type_params = match a {
                                Atomic::TNamedObject { type_params, .. } => type_params.to_vec(),
                                _ => Vec::new(),
                            };
                            (fqcn, type_params)
                        })
                    })
                {
                    let fqcn_resolved = crate::db::resolve_name(self.db, self.file.as_ref(), fqcn);
                    let fqcn_arc: Arc<str> = Arc::from(fqcn_resolved.as_str());
                    if let Some(resolved) = crate::call::method::resolve_method_from_db(
                        self.db,
                        &fqcn_arc,
                        &method_name_lower,
                    ) {
                        self.record_ref(
                            Arc::from(format!(
                                "meth:{}::{}",
                                resolved.owner_fqcn, method_name_lower
                            )),
                            method.span,
                        );
                        // Hover/go-to-definition symbol, matching the plain
                        // `$obj->method()` call form — see the Function arm above.
                        self.record_symbol(
                            method.span,
                            ReferenceKind::MethodCall {
                                class: resolved.owner_fqcn.clone(),
                                method: Arc::from(method_name.as_ref()),
                            },
                            resolved.return_ty_raw.clone(),
                        );
                        // Substitute this receiver's own bound type params (e.g.
                        // `Box<int>`'s T -> int) into the method's raw param/return
                        // types before building the callable — otherwise a
                        // first-class-callable on a generic method loses the
                        // binding that the direct-call path already applies.
                        let class_tps = crate::db::class_template_params(self.db, &fqcn_arc)
                            .map(|tps| tps.to_vec())
                            .unwrap_or_default();
                        let mut bindings =
                            crate::generic::build_class_bindings(&class_tps, &receiver_type_params);
                        let inherited_bindings =
                            crate::db::inherited_template_bindings(self.db, &fqcn_arc, &bindings);
                        // The resolved method's params/return are declared on
                        // `resolved.owner_fqcn` — own-bindings-wins only when
                        // the receiver's own class declares the method
                        // (same collision the direct-call path already
                        // guards against, see call/method.rs).
                        if resolved.owner_fqcn.as_ref() == fqcn_arc.as_ref() {
                            for (k, v) in inherited_bindings {
                                bindings.entry(k).or_insert(v);
                            }
                        } else {
                            bindings.extend(inherited_bindings);
                        }
                        let closure = Self::build_closure_from_resolved_params(
                            &resolved.params,
                            resolved.return_ty_raw,
                            &bindings,
                            &resolved.template_params,
                            &fqcn_arc,
                            &receiver_type_params,
                        );
                        return if nullsafe {
                            Type::nullable(closure)
                        } else {
                            Type::single(closure)
                        };
                    }
                    // Unlike the ordinary `$obj->method()` call path, a
                    // first-class-callable on an undefined method never raised
                    // UndefinedMethod — mirror call/method.rs's suppression
                    // rules (interface/abstract/trait receivers, __call, and
                    // an active method_exists() guard all still apply).
                    self.emit_undefined_method_for_callable(
                        &fqcn_arc,
                        &method_name,
                        &method_name_lower,
                        object,
                        ctx,
                        method.span,
                    );
                }
                Type::single(Atomic::TCallable {
                    params: None,
                    return_type: None,
                })
            }

            CallableCreateKind::StaticMethod { class, method } => {
                let method_name = match &method.kind {
                    ExprKind::Identifier(name) => name.clone(),
                    _ => {
                        // Method name isn't statically known — same coarse
                        // exemption the ordinary `Foo::$name(...)` dynamic call
                        // gets, or a private static method reachable only via
                        // `Foo::$name(...)` (first-class-callable form) is
                        // falsely flagged unused.
                        if let ExprKind::Identifier(name) = &class.kind {
                            let resolved =
                                crate::db::resolve_name(self.db, self.file.as_ref(), name.as_ref());
                            let fqcn = match crate::util::php_ident_lowercase(&resolved).as_str() {
                                "self" => ctx.self_fqcn.as_deref().unwrap_or(&resolved).to_string(),
                                "parent" => {
                                    ctx.parent_fqcn.as_deref().unwrap_or(&resolved).to_string()
                                }
                                "static" => ctx
                                    .static_fqcn
                                    .as_deref()
                                    .or(ctx.self_fqcn.as_deref())
                                    .unwrap_or(&resolved)
                                    .to_string(),
                                _ => resolved,
                            };
                            if !matches!(fqcn.as_str(), "self" | "static" | "parent") {
                                self.record_ref(Arc::from(format!("dyn:{fqcn}")), method.span);
                            }
                        } else {
                            let class_ty = self.analyze(class, ctx);
                            self.record_dynamic_member_access(&class_ty, method.span);
                        }
                        self.analyze(method, ctx);
                        return Type::single(Atomic::TCallable {
                            params: None,
                            return_type: None,
                        });
                    }
                };
                let method_name_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                let mut receiver_type_params: Vec<Type> = Vec::new();
                let is_named_class = matches!(&class.kind, ExprKind::Identifier(_));
                let fqcn = match &class.kind {
                    ExprKind::Identifier(name) => {
                        let resolved =
                            crate::db::resolve_name(self.db, self.file.as_ref(), name.as_ref());
                        match crate::util::php_ident_lowercase(&resolved).as_str() {
                            "self" => ctx.self_fqcn.as_deref().unwrap_or("self").to_string(),
                            "parent" => ctx.parent_fqcn.as_deref().unwrap_or("parent").to_string(),
                            "static" => ctx
                                .static_fqcn
                                .as_deref()
                                .unwrap_or(ctx.self_fqcn.as_deref().unwrap_or("static"))
                                .to_string(),
                            _ => resolved,
                        }
                    }
                    _ => {
                        let ty = self.analyze(class, ctx);
                        match ty.types.iter().find_map(|a| {
                            a.named_object_fqcn().map(|fqcn| {
                                let type_params = match a {
                                    Atomic::TNamedObject { type_params, .. } => {
                                        type_params.to_vec()
                                    }
                                    _ => Vec::new(),
                                };
                                (fqcn.to_string(), type_params)
                            })
                        }) {
                            Some((fqcn, type_params)) => {
                                receiver_type_params = type_params;
                                fqcn
                            }
                            None => {
                                return Type::single(Atomic::TCallable {
                                    params: None,
                                    return_type: None,
                                })
                            }
                        }
                    }
                };
                // A named class token (`Foo::bar(...)`) must resolve to a real
                // class — unlike the object-derived branch above, whose fqcn
                // already came from a resolved (thus existing) receiver type.
                // Without this check an undefined class here silently produced
                // a generic `TCallable` with no diagnostic at all, unlike the
                // identical `Foo::bar()` direct-call form.
                if is_named_class
                    && !matches!(fqcn.as_str(), "self" | "static" | "parent")
                    && !crate::db::class_exists(self.db, &fqcn)
                    && !ctx.is_class_guarded(fqcn.as_str())
                {
                    self.emit(
                        IssueKind::UndefinedClass { name: fqcn },
                        Severity::Error,
                        class.span,
                    );
                    return Type::single(Atomic::TCallable {
                        params: None,
                        return_type: None,
                    });
                }
                let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
                if is_named_class {
                    self.record_ref(Arc::from(format!("cls:{fqcn_arc}")), class.span);
                }
                if let Some(resolved) = crate::call::method::resolve_method_from_db(
                    self.db,
                    &fqcn_arc,
                    &method_name_lower,
                ) {
                    self.record_ref(
                        Arc::from(format!(
                            "meth:{}::{}",
                            resolved.owner_fqcn, method_name_lower
                        )),
                        method.span,
                    );
                    // Hover/go-to-definition symbol, matching the plain
                    // `Foo::method()` call form — see the Function arm above.
                    self.record_symbol(
                        method.span,
                        ReferenceKind::StaticCall {
                            class: resolved.owner_fqcn.clone(),
                            method: Arc::from(method_name.as_ref()),
                        },
                        resolved.return_ty_raw.clone(),
                    );
                    // Same reasoning as the instance-method FCC case above: substitute
                    // the receiver's own bound type params before building the callable.
                    let class_tps = crate::db::class_template_params(self.db, &fqcn_arc)
                        .map(|tps| tps.to_vec())
                        .unwrap_or_default();
                    let mut bindings =
                        crate::generic::build_class_bindings(&class_tps, &receiver_type_params);
                    let inherited_bindings =
                        crate::db::inherited_template_bindings(self.db, &fqcn_arc, &bindings);
                    // Own-bindings-wins only when the receiver's own class
                    // declares the method (`resolved.owner_fqcn`); otherwise
                    // the ancestor that actually declares it wins — same
                    // collision guard as the instance-method FCC case above.
                    if resolved.owner_fqcn.as_ref() == fqcn_arc.as_ref() {
                        for (k, v) in inherited_bindings {
                            bindings.entry(k).or_insert(v);
                        }
                    } else {
                        bindings.extend(inherited_bindings);
                    }
                    return Type::single(Self::build_closure_from_resolved_params(
                        &resolved.params,
                        resolved.return_ty_raw,
                        &bindings,
                        &resolved.template_params,
                        &fqcn_arc,
                        &receiver_type_params,
                    ));
                }
                self.emit_undefined_static_method_for_callable(
                    &fqcn_arc,
                    &method_name,
                    method.span,
                );
                Type::single(Atomic::TCallable {
                    params: None,
                    return_type: None,
                })
            }
        }
    }

    /// UndefinedMethod check for a first-class-callable (`$obj->method(...)`)
    /// whose method didn't resolve — mirrors call/method.rs's ordinary-call
    /// suppression rules (interface/abstract/trait receivers, `__call`, and an
    /// active `method_exists()` guard), which the FCC path previously never ran.
    fn emit_undefined_method_for_callable(
        &mut self,
        fqcn: &Arc<str>,
        method_name: &str,
        method_name_lower: &str,
        object: &php_ast::owned::Expr,
        ctx: &FlowState,
        span: php_ast::Span,
    ) {
        if !crate::db::class_exists(self.db, fqcn) || crate::db::has_unknown_ancestor(self.db, fqcn)
        {
            return;
        }
        let (is_interface, is_abstract, is_trait) = crate::db::class_kind(self.db, fqcn)
            .map(|k| (k.is_interface, k.is_abstract, k.is_trait))
            .unwrap_or((false, false, false));
        let has_call_magic = crate::db::has_method_in_chain(self.db, fqcn, "__call");
        let guarded_by_method_exists =
            crate::narrowing::extract_expr_guard_key(object, ctx, self.db, &self.file)
                .map(|key| {
                    ctx.method_exists_guards
                        .contains(&(key, Arc::from(method_name_lower)))
                })
                .unwrap_or(false);
        if is_interface || is_abstract || is_trait || has_call_magic || guarded_by_method_exists {
            return;
        }
        self.emit(
            IssueKind::UndefinedMethod {
                class: fqcn.to_string(),
                method: method_name.to_string(),
            },
            Severity::Error,
            span,
        );
    }

    /// Same as `emit_undefined_method_for_callable`, for the static-call form
    /// (`Foo::method(...)`) — mirrors call/static_call.rs's suppression rules
    /// (abstract/trait receivers and `__callStatic`; no `method_exists()` guard,
    /// since that pattern doesn't apply to static calls).
    fn emit_undefined_static_method_for_callable(
        &mut self,
        fqcn: &Arc<str>,
        method_name: &str,
        span: php_ast::Span,
    ) {
        if !crate::db::class_exists(self.db, fqcn) || crate::db::has_unknown_ancestor(self.db, fqcn)
        {
            return;
        }
        let (is_abstract, is_trait) = crate::db::class_kind(self.db, fqcn)
            .map(|k| (k.is_abstract, k.is_trait))
            .unwrap_or((false, false));
        let has_callstatic_magic = crate::db::has_method_in_chain(self.db, fqcn, "__callstatic");
        if is_abstract || is_trait || has_callstatic_magic {
            return;
        }
        self.emit(
            IssueKind::UndefinedMethod {
                class: fqcn.to_string(),
                method: method_name.to_string(),
            },
            Severity::Error,
            span,
        );
    }

    fn build_closure_from_resolved_params(
        params: &[mir_codebase::definitions::DeclaredParam],
        return_ty: Type,
        bindings: &rustc_hash::FxHashMap<mir_types::Name, Type>,
        own_template_params: &[mir_codebase::definitions::TemplateParam],
        receiver_fqcn: &Arc<str>,
        receiver_type_params: &[Type],
    ) -> Atomic {
        // A method-level `@template` SHADOWS a same-named class template, so it
        // must not get the class-level binding baked in here. Unlike a direct
        // call (where `check_args` sees the real arguments and infers+bound-checks
        // the method's own templates per call), a first-class-callable value is
        // built once and may be called anywhere later with no template_params in
        // scope to infer against (see the `template_params: &[]` call to
        // `check_args` for a closure-typed callee) — so a bare, still-templated
        // param would silently accept any argument. Substituting each of the
        // method's own templates with its declared bound (or `mixed` if
        // unbounded, the same fallback `infer_template_bindings` uses) instead
        // keeps every call through the closure checked against at least that
        // bound, which is always sound: an argument to the real method must
        // satisfy the bound regardless of what the template later infers to.
        let mut bindings = bindings.clone();
        for tp in own_template_params {
            let bound_ty = tp.bound.as_deref().cloned().unwrap_or_else(Type::mixed);
            bindings.insert(mir_types::Name::from(tp.name.as_ref()), bound_ty);
        }
        let resolve = |t: Type| -> Type {
            // `self`/`static` must resolve to the receiver's concrete class, the
            // same way a direct call's return type already does — otherwise a
            // late-static-bound method's FCC closure carries the class that
            // physically declares the method instead of the real receiver.
            crate::call::substitute_static_in_return(t, receiver_fqcn, receiver_type_params)
                .substitute_templates(&bindings)
        };
        let fn_params: Box<[mir_types::atomic::FnParam]> = params
            .iter()
            .map(|p| mir_types::atomic::FnParam {
                name: mir_types::Name::from(p.name.as_ref()),
                ty: p
                    .ty
                    .as_deref()
                    .cloned()
                    .map(resolve)
                    .map(mir_types::compact::SimpleType::from_union),
                out_ty: p
                    .out_ty
                    .as_deref()
                    .cloned()
                    .map(resolve)
                    .map(mir_types::compact::SimpleType::from_union),
                default: if p.has_default {
                    Some(mir_types::compact::SimpleType::from_union(Type::mixed()))
                } else {
                    None
                },
                is_variadic: p.is_variadic,
                is_byref: p.is_byref,
                is_optional: p.is_optional,
            })
            .collect();
        Atomic::TClosure {
            data: Box::new(mir_types::atomic::ClosureData {
                params: fn_params,
                return_type: resolve(return_ty),
                this_type: None,
            }),
        }
    }

    /// Record a reference location for `symbol_key` at `span`, unless in inference-only mode.
    pub(crate) fn record_ref(&self, symbol_key: Arc<str>, span: php_ast::Span) {
        if self.mode == AnalysisMode::InferenceOnly {
            return;
        }
        // Static property tokens (`Cls::$prop`) span the `$` sigil while
        // instance accesses (`$obj->prop`) don't — normalize property spans
        // to the bare name so find-references ranges are uniform. Global
        // constant tokens may span a qualified path (`\Config\DB_HOST`);
        // narrow to the final segment for the same reason.
        let mut start = span.start;
        if symbol_key.starts_with("prop:")
            && self.source.as_bytes().get(start as usize) == Some(&b'$')
        {
            start += 1;
        }
        if symbol_key.starts_with("gcnst:") {
            let s = start as usize;
            let e = (span.end as usize).min(self.source.len());
            if let Some(slice) = self.source.get(s..e) {
                if let Some(pos) = slice.rfind('\\') {
                    start += pos as u32 + 1;
                }
            }
        }
        let (line, col_start) = self.offset_to_line_col(start);
        let (_, col_end) = self.offset_to_line_col(span.end);
        self.db.record_reference_location(crate::db::RefLoc {
            symbol_key,
            file: self.file.clone(),
            line,
            col_start,
            col_end,
        });
    }

    /// Mark every named-object class in `obj_ty` as reached through a
    /// dynamic member access (`$obj->$name`/`$obj->$name()` where `$name`
    /// isn't a literal). The exact member touched is unknowable statically,
    /// so instead of under- or over-recording a specific reference, this
    /// records a coarse per-class marker (`dyn:Fqcn`) that
    /// `DeadCodeAnalyzer` consults to blanket-exempt a class's private
    /// members from `Unused*` once any dynamic access on it is seen —
    /// otherwise a private member reachable only dynamically (a common
    /// `__get`/`__set`-adjacent pattern) is falsely reported unused.
    pub(crate) fn record_dynamic_member_access(&self, obj_ty: &Type, span: php_ast::Span) {
        if self.mode == AnalysisMode::InferenceOnly {
            return;
        }
        for atomic in obj_ty.remove_null().types.iter() {
            let fqcn = atomic.named_object_fqcn().or_else(|| match atomic {
                Atomic::TClassString(Some(fqcn)) => Some(fqcn.as_ref()),
                _ => None,
            });
            if let Some(fqcn) = fqcn {
                self.record_ref(Arc::from(format!("dyn:{fqcn}")), span);
            }
        }
    }

    /// Walk a type hint and emit `UndefinedClass` for any named type not in the codebase.
    fn check_type_hint(&mut self, hint: &php_ast::owned::TypeHint) {
        use php_ast::owned::TypeHintKind;
        match &hint.kind {
            TypeHintKind::Named(name) => {
                let name_str = crate::parser::name_to_string_owned(name);
                if matches!(
                    crate::util::php_ident_lowercase(&name_str).as_str(),
                    "self"
                        | "static"
                        | "parent"
                        | "null"
                        | "true"
                        | "false"
                        | "never"
                        | "void"
                        | "mixed"
                        | "object"
                        | "callable"
                        | "iterable"
                ) {
                    return;
                }
                let resolved = crate::db::resolve_name(self.db, &self.file, &name_str);
                if !crate::db::class_exists(self.db, &resolved) {
                    self.emit(
                        IssueKind::UndefinedClass { name: resolved },
                        Severity::Error,
                        hint.span,
                    );
                } else {
                    let fqcn: Arc<str> = Arc::from(resolved.as_str());
                    self.record_ref(Arc::from(format!("cls:{fqcn}")), hint.span);
                    self.record_symbol(
                        hint.span,
                        ReferenceKind::ClassReference(fqcn),
                        mir_types::Type::single(mir_types::Atomic::TClassString(None)),
                    );
                }
            }
            TypeHintKind::Nullable(inner) => self.check_type_hint(inner),
            TypeHintKind::Union(parts) | TypeHintKind::Intersection(parts) => {
                for part in parts.iter() {
                    self.check_type_hint(part);
                }
            }
            TypeHintKind::Keyword(_, _) => {}
        }
    }

    /// Convert plugin-raised issues into real diagnostics at their spans.
    pub(crate) fn emit_plugin_issues(
        &mut self,
        issues: Vec<mir_plugin::PluginIssue>,
        default_span: php_ast::Span,
    ) {
        for pi in issues {
            let span = pi.span.unwrap_or(default_span);
            self.emit(
                IssueKind::PluginIssue {
                    name: pi.name,
                    message: pi.message,
                },
                pi.severity,
                span,
            );
        }
    }

    /// Turn a plugin-provided type into a concrete `Type`. `Parse` strings go
    /// through the docblock type parser and are namespace-resolved against
    /// the current file, so `SomeClass` from a plugin matches the codebase's
    /// stored FQCNs.
    pub(crate) fn resolve_provided_type(&self, provided: mir_plugin::ProvidedType) -> Type {
        match provided {
            mir_plugin::ProvidedType::Union(t) => t,
            mir_plugin::ProvidedType::Parse(s) => {
                let raw = crate::parser::docblock::parse_type_string(&s);
                crate::stmt::resolve_union_for_file(raw, self.db, &self.file)
            }
        }
    }

    /// Run function-call plugin hooks: return-type providers first (a
    /// provider result replaces the inferred return type, mirroring Psalm's
    /// `FunctionReturnTypeProviderInterface`), then `AfterFunctionCallAnalysis`.
    pub(crate) fn apply_function_call_plugins(
        &mut self,
        fqn: &str,
        args: &[php_ast::owned::Arg],
        arg_types: &[Type],
        span: php_ast::Span,
        return_ty: &mut Type,
    ) {
        let Some(plugins) = self.plugins.clone() else {
            return;
        };
        let has_after = plugins.hooks().after_function_call_analysis;
        if !plugins.has_any_function_provider() && !has_after {
            return;
        }
        let function_id = mir_plugin::normalize_id(fqn);
        let file = self.file.clone();
        if plugins.has_function_provider(&function_id) {
            let snippet = crate::parser::span_text(self.source, span);
            let event = mir_plugin::FunctionReturnTypeProviderEvent {
                function_id: &function_id,
                args,
                arg_types,
                span,
                file: file.as_ref(),
                call_snippet: snippet.as_deref(),
            };
            if let Some(provided) = plugins.function_return_type(&event) {
                *return_ty = self.resolve_provided_type(provided);
            }
        }
        if has_after {
            let mut event = mir_plugin::AfterFunctionCallAnalysisEvent {
                function_id: &function_id,
                args,
                arg_types,
                span,
                file: file.as_ref(),
                return_type: return_ty,
                issues: Vec::new(),
            };
            plugins.after_function_call_analysis(&mut event);
            let issues = std::mem::take(&mut event.issues);
            drop(event);
            self.emit_plugin_issues(issues, span);
        }
    }

    /// Run method-call plugin hooks. Providers are matched against the
    /// receiver class first, then the declaring class, mirroring how Psalm
    /// dispatches `MethodReturnTypeProviderInterface` up the hierarchy.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn apply_method_call_plugins(
        &mut self,
        receiver_fqcn: &str,
        owner_fqcn: &str,
        method_name: &str,
        args: &[php_ast::owned::Arg],
        arg_types: &[Type],
        span: php_ast::Span,
        return_ty: &mut Type,
    ) {
        let Some(plugins) = self.plugins.clone() else {
            return;
        };
        let has_after = plugins.hooks().after_method_call_analysis;
        if !plugins.has_any_method_provider() && !has_after {
            return;
        }
        let method_lower = method_name.to_ascii_lowercase();
        let file = self.file.clone();
        let snippet = crate::parser::span_text(self.source, span);
        let mut candidates = vec![receiver_fqcn];
        if !owner_fqcn.eq_ignore_ascii_case(receiver_fqcn) {
            candidates.push(owner_fqcn);
        }
        for fqcn in candidates {
            let normalized = mir_plugin::normalize_id(fqcn);
            if !plugins.has_method_provider(&normalized) {
                continue;
            }
            let event = mir_plugin::MethodReturnTypeProviderEvent {
                fqcn,
                method_name: &method_lower,
                args,
                arg_types,
                span,
                file: file.as_ref(),
                call_snippet: snippet.as_deref(),
            };
            if let Some(provided) = plugins.method_return_type(&normalized, &event) {
                *return_ty = self.resolve_provided_type(provided);
                break;
            }
        }
        if has_after {
            let method_id = format!("{receiver_fqcn}::{method_lower}");
            let mut event = mir_plugin::AfterMethodCallAnalysisEvent {
                method_id: &method_id,
                args,
                arg_types,
                span,
                file: file.as_ref(),
                return_type: return_ty,
                issues: Vec::new(),
            };
            plugins.after_method_call_analysis(&mut event);
            let issues = std::mem::take(&mut event.issues);
            drop(event);
            self.emit_plugin_issues(issues, span);
        }
    }

    /// Consult class-property providers for an otherwise-undeclared property.
    /// Fires when the receiver's own class or any ancestor is registered as a
    /// class-property marker (Psalm's `PropertiesProviderInterface` shape), so
    /// a framework base class covers every user subclass. Returns the provided
    /// type, or `None` to fall through to normal `UndefinedProperty` reporting.
    pub(crate) fn class_property_from_plugin(&self, fqcn: &str, prop_name: &str) -> Option<Type> {
        let plugins = self.plugins.clone()?;
        if !plugins.has_any_class_property_provider() {
            return None;
        }
        let here = crate::db::Fqcn::from_str(self.db, fqcn);
        let chain = crate::db::class_ancestors_by_fqcn(self.db, here);
        let matched: Vec<String> = chain
            .iter()
            .map(|c| mir_plugin::normalize_id(c))
            .filter(|n| plugins.has_class_property_marker(n))
            .collect();
        if matched.is_empty() {
            return None;
        }
        // Aggregate array-literal property defaults along the chain,
        // nearest-class-wins, so a subclass `$casts` shadows a base class's.
        let mut defaults: Vec<mir_plugin::ArrayPropertyDefault> = Vec::new();
        for c in chain.iter() {
            let cf = crate::db::Fqcn::from_str(self.db, c);
            for d in crate::db::class_array_property_defaults(self.db, cf).iter() {
                if !defaults.iter().any(|e| e.property == d.property) {
                    defaults.push(d.clone());
                }
            }
        }
        let event = mir_plugin::ClassPropertyProviderEvent {
            fqcn,
            property_name: prop_name,
            array_property_defaults: &defaults,
            file: self.file.as_ref(),
        };
        for marker in &matched {
            if let Some(provided) = plugins.class_property(marker, &event) {
                return Some(self.resolve_provided_type(provided));
            }
        }
        None
    }

    pub fn emit(&mut self, kind: IssueKind, severity: Severity, span: php_ast::Span) {
        let (line, col_start) = self.offset_to_line_col(span.start);

        let (line_end, col_end) = if span.start < span.end {
            let (end_line, end_col) = self.offset_to_line_col(span.end);
            (end_line, end_col)
        } else {
            (line, col_start)
        };

        let mut issue = Issue::new(
            kind,
            Location {
                file: self.file.clone(),
                line,
                line_end,
                col_start,
                col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
            },
        );
        issue.severity = severity;
        // Store the source snippet for baseline matching.
        if span.start < span.end {
            let s = span.start as usize;
            let e = (span.end as usize).min(self.source.len());
            if let Some(text) = self.source.get(s..e) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    issue.snippet = Some(trimmed.to_string());
                }
            }
        }
        self.issues.add(issue);
    }

    /// Emit a clone diagnostic when `ty` is (possibly) not an object. `mixed`
    /// takes precedence (matching the historical `MixedClone` behaviour), then
    /// definite non-objects (`InvalidClone`) and mixed object/non-object unions
    /// (`PossiblyInvalidClone`).
    fn check_clone_target(&mut self, ty: &Type, span: php_ast::Span) {
        if ty.is_mixed() {
            self.emit(IssueKind::MixedClone, Severity::Info, span);
            return;
        }
        match ty.clone_validity() {
            CloneValidity::Invalid => self.emit(
                IssueKind::InvalidClone { ty: ty.to_string() },
                Severity::Error,
                span,
            ),
            CloneValidity::PossiblyInvalid => self.emit(
                IssueKind::PossiblyInvalidClone { ty: ty.to_string() },
                Severity::Info,
                span,
            ),
            CloneValidity::Cloneable | CloneValidity::Unknown => {}
        }
    }

    /// Emit DeprecatedMethodCall if the cloned object has a deprecated __clone() method.
    fn check_clone_deprecated(&mut self, ty: &Type, span: php_ast::Span) {
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if let Some((_, method)) = crate::db::find_method_in_chain(
                    self.db,
                    crate::db::Fqcn::from_str(self.db, fqcn_str),
                    "__clone",
                ) {
                    if let Some(msg) = &method.deprecated {
                        self.emit(
                            IssueKind::DeprecatedMethodCall {
                                class: fqcn_str.to_string(),
                                method: "__clone".to_string(),
                                message: Some(msg.clone()).filter(|m| !m.is_empty()),
                            },
                            Severity::Info,
                            span,
                        );
                    }
                }
            }
        }
    }

    /// Emit InvalidClone if the object's __clone() is private and the current
    /// class context doesn't have access to it.
    /// Only runs when the type union is purely cloneable (not already flagged
    /// by check_clone_target for non-object members).
    fn check_clone_visibility(
        &mut self,
        ty: &Type,
        span: php_ast::Span,
        caller_fqcn: Option<&str>,
    ) {
        use mir_codebase::definitions::Visibility;
        use mir_types::CloneValidity;
        // Skip if the type already has structural clone issues (non-object components)
        match ty.clone_validity() {
            CloneValidity::Invalid | CloneValidity::PossiblyInvalid => return,
            _ => {}
        }
        for atomic in &ty.types {
            if let Atomic::TNamedObject { fqcn, .. } = atomic {
                let fqcn_str = fqcn.as_ref();
                if let Some((owner_fqcn, method)) = crate::db::find_method_in_chain(
                    self.db,
                    crate::db::Fqcn::from_str(self.db, fqcn_str),
                    "__clone",
                ) {
                    if method.visibility == Visibility::Private {
                        // Private __clone is accessible only from within the declaring class
                        let accessible = caller_fqcn
                            .map(|c| c.eq_ignore_ascii_case(owner_fqcn.as_ref()))
                            .unwrap_or(false);
                        if !accessible {
                            self.emit(
                                IssueKind::InvalidClone {
                                    ty: fqcn_str.to_string(),
                                },
                                Severity::Error,
                                span,
                            );
                        }
                    }
                }
            }
        }
    }

    fn check_interpolation_implicit_to_string_cast(&mut self, ty: &Type, span: php_ast::Span) {
        for atomic in &ty.types {
            // Mirrors expr::binary's implicit-to-string check: a non-Stringable
            // enum is just as fatal to `print`/interpolation as any other bare
            // object.
            let fqcn = match atomic {
                Atomic::TNamedObject { fqcn, .. } => Some(fqcn),
                Atomic::TLiteralEnumCase { enum_fqcn, .. } => Some(enum_fqcn),
                _ => None,
            };
            let Some(fqcn) = fqcn else { continue };
            let fqcn_str = fqcn.as_ref();
            if !crate::db::has_method_in_chain(self.db, fqcn_str, "__toString")
                && !crate::db::extends_or_implements(self.db, fqcn_str, "Stringable")
            {
                self.emit(
                    IssueKind::ImplicitToStringCast {
                        class: fqcn_str.to_string(),
                    },
                    Severity::Warning,
                    span,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    /// Helper to create a SourceMap from PHP source code
    fn create_source_map(source: &str) -> php_rs_parser::source_map::SourceMap {
        php_rs_parser::parse(source).source_map
    }

    /// Helper to test offset_to_line_col conversion (Unicode char-count columns).
    fn test_offset_conversion(source: &str, offset: u32) -> (u32, u16) {
        let source_map = create_source_map(source);
        let lc = source_map.offset_to_line_col(offset);
        let line = lc.line + 1;

        let byte_offset = offset as usize;
        let line_start_byte = if byte_offset == 0 {
            0
        } else {
            source[..byte_offset]
                .rfind('\n')
                .map(|p| p + 1)
                .unwrap_or(0)
        };

        let col = source[line_start_byte..byte_offset].chars().count() as u16;

        (line, col)
    }

    #[test]
    fn col_conversion_simple_ascii() {
        let source = "<?php\n$var = 123;";

        // '$' on line 2, column 0
        let (line, col) = test_offset_conversion(source, 6);
        assert_eq!(line, 2);
        assert_eq!(col, 0);

        // 'v' on line 2, column 1
        let (line, col) = test_offset_conversion(source, 7);
        assert_eq!(line, 2);
        assert_eq!(col, 1);
    }

    #[test]
    fn col_conversion_different_lines() {
        let source = "<?php\n$x = 1;\n$y = 2;";
        // Line 1: <?php     (bytes 0-4, newline at 5)
        // Line 2: $x = 1;  (bytes 6-12, newline at 13)
        // Line 3: $y = 2;  (bytes 14-20)

        let (line, col) = test_offset_conversion(source, 0);
        assert_eq!((line, col), (1, 0));

        let (line, col) = test_offset_conversion(source, 6);
        assert_eq!((line, col), (2, 0));

        let (line, col) = test_offset_conversion(source, 14);
        assert_eq!((line, col), (3, 0));
    }

    #[test]
    fn col_conversion_accented_characters() {
        // é is 2 UTF-8 bytes but 1 Unicode char (and 1 UTF-16 unit — same result either way)
        let source = "<?php\n$café = 1;";
        // Line 2: $ c a f é ...
        // bytes:  6 7 8 9 10(2 bytes)

        // 'f' at byte 9 → char col 3
        let (line, col) = test_offset_conversion(source, 9);
        assert_eq!((line, col), (2, 3));

        // 'é' at byte 10 → char col 4
        let (line, col) = test_offset_conversion(source, 10);
        assert_eq!((line, col), (2, 4));
    }

    #[test]
    fn col_conversion_emoji_counts_as_one_char() {
        // 🎉 (U+1F389) is 4 UTF-8 bytes and 2 UTF-16 units, but 1 Unicode char.
        // A char after the emoji must land at col 7, not col 8.
        let source = "<?php\n$y = \"🎉x\";";
        // Line 2: $ y   =   " 🎉 x " ;
        // chars:  0 1 2 3 4 5  6  7 8 9

        let emoji_start = source.find("🎉").unwrap();
        let after_emoji = emoji_start + "🎉".len(); // skip 4 bytes

        // position at 'x' (right after the emoji)
        let (line, col) = test_offset_conversion(source, after_emoji as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 7); // emoji counts as 1, not 2
    }

    #[test]
    fn col_conversion_emoji_start_position() {
        // The opening quote is at col 5; the emoji immediately follows at col 6.
        let source = "<?php\n$y = \"🎉\";";
        // Line 2: $ y   =   " 🎉 " ;
        // chars:  0 1 2 3 4 5  6  7 8

        let quote_pos = source.find('"').unwrap();
        let emoji_pos = quote_pos + 1; // byte after opening quote = emoji start

        let (line, col) = test_offset_conversion(source, quote_pos as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 5); // '"' is the 6th char on line 2 (0-based: col 5)

        let (line, col) = test_offset_conversion(source, emoji_pos as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 6); // emoji follows the quote
    }

    #[test]
    fn col_end_minimum_width() {
        // Same-line span: ensure col_end is at least col_start + 1 (1 character minimum).
        let effective_col_end = crate::diagnostics::clamp_col_end(1, 1, 0, 0);
        assert_eq!(
            effective_col_end, 1,
            "col_end should be at least col_start + 1 on the same line"
        );
    }

    #[test]
    fn col_end_not_clamped_across_lines() {
        // A multi-line span (e.g. a `new Foo(\n ...\n)` call) legitimately has
        // col_end smaller than col_start, since they're columns on different
        // lines. Regression for a bug where `.max(col_start + 1)` was applied
        // unconditionally, producing a nonsensical column past the end of the
        // end line's actual content (see `TooManyArguments` on 0-arg `new` calls).
        let col_start = 11u16; // e.g. "    return new Foo(" — "new" starts at col 11
        let col_end = 5u16; // e.g. "    );" — ")" ends at col 5
        let effective_col_end = crate::diagnostics::clamp_col_end(1, 8, col_start, col_end);

        assert_eq!(
            effective_col_end, col_end,
            "col_end on a different line must not be clamped up to col_start + 1"
        );
    }

    #[test]
    fn col_conversion_multiline_span() {
        // Test span that starts on one line and ends on another
        let source = "<?php\n$x = [\n  'a',\n  'b'\n];";
        //           Line 1: <?php
        //           Line 2: $x = [
        //           Line 3:   'a',
        //           Line 4:   'b'
        //           Line 5: ];

        // Start of array bracket on line 2
        let bracket_open = source.find('[').unwrap();
        let (line_start, _col_start) = test_offset_conversion(source, bracket_open as u32);
        assert_eq!(line_start, 2);

        // End of array bracket on line 5
        let bracket_close = source.rfind(']').unwrap();
        let (line_end, col_end) = test_offset_conversion(source, bracket_close as u32);
        assert_eq!(line_end, 5);
        assert_eq!(col_end, 0); // ']' is at column 0 on line 5
    }

    #[test]
    fn col_end_handles_emoji_in_span() {
        // Test that col_end correctly handles emoji spanning
        let source = "<?php\n$greeting = \"Hello 🎉\";";

        // Find emoji position
        let emoji_pos = source.find('🎉').unwrap();
        let hello_pos = source.find("Hello").unwrap();

        // Column at "Hello" on line 2
        let (line, col) = test_offset_conversion(source, hello_pos as u32);
        assert_eq!(line, 2);
        assert_eq!(col, 13); // Position of 'H' after "$greeting = \""

        // Column at emoji
        let (line, col) = test_offset_conversion(source, emoji_pos as u32);
        assert_eq!(line, 2);
        // Should be after "Hello " (13 + 5 + 1 = 19 chars)
        assert_eq!(col, 19);
    }
}
