use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::FunctionRefExt as _;
/// Expression analyzer — infers the `Type` type of any PHP expression.
use crate::ast_ext::NameBareExt as _;
use crate::ast_ext::NameRefExt as _;
use crate::ast_ext::NameResolveExt as _;
use std::sync::Arc;

use php_ast::owned::ExprKind;

use mir_issues::{Issue, IssueBuffer, IssueKind, Location, Severity};
use mir_types::{Atomic, CloneValidity, Type};

use crate::body_analysis::AnalysisMode;
use crate::db::MirDatabase;
use crate::flow_state::FlowState;
use crate::php_version::PhpVersion;
use crate::reference_key::ReferenceKeyCache;
use crate::symbol::{NavigationFact, ReferenceKind, ResolvedNavigationFact, ResolvedSymbol};

mod arrays;
pub(crate) mod assignment;
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

pub(crate) use assignment::root_receiver_var;
pub(crate) use binary::{operand_is_definitely_zero, pipe_as_call};
pub use helpers::{duplicate_literal_conditions, extract_simple_var};

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
    pub symbols: Option<&'a mut Vec<ResolvedSymbol>>,
    pub navigation_facts: &'a std::cell::RefCell<Vec<NavigationFact>>,
    pub resolved_navigation_facts: &'a std::cell::RefCell<Vec<ResolvedNavigationFact>>,
    pub php_version: PhpVersion,
    pub mode: AnalysisMode,
    /// Whether `declare(strict_types=1)` is active for the calling file.
    /// When true, coercive PHP typing (e.g. Stringable → string) must not be
    /// silently allowed — the runtime would throw a TypeError.
    pub strict_types: bool,
    /// When false, `record_symbol*` calls are no-ops — see
    /// `StatementsAnalyzer::collect_symbols`.
    pub collect_symbols: bool,
    /// When false, recorded symbols keep only navigation shape and use
    /// `Type::mixed()` instead of cloning the precise resolved type.
    pub capture_symbol_types: bool,
    /// When true, skip symbols that cannot map to a codebase-level
    /// [`crate::Name`] (currently variables and receiver-gap markers).
    pub codebase_symbols_only: bool,
    /// When false, skip recording reference-location postings.
    pub record_reference_locations: bool,
    /// When true, collect compact codebase-name navigation facts.
    pub collect_navigation_facts: bool,
    /// When true, collect compact typed facts for codebase symbol resolution.
    pub collect_resolved_navigation_facts: bool,
    /// When true, we are inside an existence-check context (isset/empty/??) where missing
    /// variables and missing array offsets are not errors — they are what is being tested.
    in_existence_check: bool,
    /// True while analyzing the array access bound by `=&`, where a null base auto-vivifies.
    vivifying_reference: bool,
    /// Param types the callee passes to the closure literal about to be analyzed.
    pub(crate) callback_param_hints: Option<Vec<Option<Type>>>,
    /// When true, the variable currently being analyzed is the BASE of an
    /// `ArrayAccess` expression (`$_SERVER` inside `$_SERVER['x']`), which
    /// already runs its own superglobal purity check at the `ArrayAccess`
    /// level (`arrays.rs::analyze_array_access`) keyed to the whole
    /// expression's span — without this, `analyze_variable`'s own bare-read
    /// superglobal check would fire a SECOND time for the same access, at a
    /// different (narrower) span, a genuine double-report rather than a
    /// harmlessly-deduplicated one.
    in_array_access_base: bool,
    /// `(key type, value type)` for every `yield`/`yield from` encountered so
    /// far in the enclosing function body, regardless of which branch/loop it
    /// syntactically sits in — the AST is walked once, so no branch-merge
    /// logic is needed. Borrowed from the owning `StatementsAnalyzer` so it
    /// survives across the many short-lived `ExpressionAnalyzer`s created
    /// while walking one function body. Read back by `body_analysis` to
    /// build the function's inferred `Generator<K, V, S, R>` return type.
    pub(crate) yielded_types: &'a mut Vec<(Type, Type)>,
    /// Borrowed from the owning `StatementsAnalyzer`, like `yielded_types`.
    pub(crate) taint_summary: &'a mut crate::taint::TaintSummary,
    /// Snapshot of the installed plugin registry (`None` when no plugins are
    /// loaded — the common case, making every hook site a single check).
    pub(crate) plugins: Option<std::sync::Arc<mir_plugin::PluginRegistry>>,
    pub(crate) reference_key_cache: &'a mut ReferenceKeyCache,
}

/// A first-class-callable creation expression (`foo(...)`, `$x(...)`,
/// `$obj->method(...)`, `Foo::method(...)`) always evaluates to a `Closure`
/// instance, regardless of what the callee is — used as the fallback type
/// wherever `callable_create_type` can't statically resolve a signature to
/// build a fully-typed `TClosure` from (dynamic callee, dynamic method name,
/// unresolvable receiver/class, undefined method/class, ...).
fn bare_closure() -> Atomic {
    Atomic::TNamedObject {
        fqcn: mir_types::Name::from("Closure"),
        type_params: mir_types::union::empty_type_params(),
    }
}

fn is_partial_application(args: &[php_ast::owned::Arg]) -> bool {
    args.iter().any(|a| a.is_placeholder())
}

/// A call with `?` placeholders creates a closure instead of running the callee (PHP 8.6).
fn partial_application_type(
    args: &[php_ast::owned::Arg],
    called: Type,
    callee: impl FnOnce() -> Option<Atomic>,
) -> Type {
    if !is_partial_application(args) {
        return called;
    }
    Type::single(partial_closure(args, callee))
}

fn partial_closure(
    args: &[php_ast::owned::Arg],
    callee: impl FnOnce() -> Option<Atomic>,
) -> Atomic {
    callee()
        .and_then(|closure| bind_placeholders(closure, args))
        .unwrap_or_else(bare_closure)
}

/// The callee's closure with only the parameters the placeholders leave open.
fn bind_placeholders(closure: Atomic, args: &[php_ast::owned::Arg]) -> Option<Atomic> {
    use php_ast::owned::ArgKind;
    let Atomic::TClosure { mut data } = closure else {
        return None;
    };
    let params = std::mem::take(&mut data.params).into_vec();
    let mut open = Vec::new();
    let mut bound = vec![false; params.len()];
    let mut position = 0;
    let mut rest = false;
    for arg in args {
        if let Some(name) = &arg.name {
            let name = crate::parser::name_to_string_owned(name);
            let index = params
                .iter()
                .position(|p| !p.is_variadic && p.name.as_str() == name)?;
            bound[index] = true;
            if matches!(arg.kind, ArgKind::Placeholder) {
                open.push(params[index].clone());
            }
            continue;
        }
        let index = match params.get(position) {
            Some(_) => position,
            None if params.last().is_some_and(|p| p.is_variadic) => params.len() - 1,
            None if matches!(arg.kind, ArgKind::SpreadPlaceholder) => position,
            None => return None,
        };
        match arg.kind {
            ArgKind::SpreadPlaceholder => {
                rest = true;
                break;
            }
            ArgKind::Spread(_) => return None,
            ArgKind::Placeholder => {
                let mut param = params[index].clone();
                param.is_variadic = false;
                param.is_optional = false;
                open.push(param);
            }
            _ => {}
        }
        if !params[index].is_variadic {
            bound[index] = true;
        }
        position += 1;
    }
    if rest {
        open.extend(
            params
                .iter()
                .zip(&bound)
                .enumerate()
                .filter(|(i, (p, bound))| !**bound && (*i >= position || p.is_variadic))
                .map(|(_, (p, _))| p.clone()),
        );
    }
    data.params = open.into_boxed_slice();
    Some(Atomic::TClosure { data })
}

impl<'a> ExpressionAnalyzer<'a> {
    /// A call that can run the class's own code sees the properties as they are now.
    fn expose_props_to_call(
        &mut self,
        target: &php_ast::owned::Expr,
        args: &[php_ast::owned::Arg],
        ctx: &FlowState,
    ) {
        let is_this =
            |e: &php_ast::owned::Expr| matches!(&e.kind, ExprKind::Variable(v) if &**v == "this");
        let own_class = is_this(target)
            || target.kind.name_bare().is_some_and(|n| {
                ["self", "static", "parent"]
                    .iter()
                    .any(|k| n.eq_ignore_ascii_case(k))
            });
        if own_class || args.iter().any(|a| a.value().is_some_and(is_this)) {
            self.taint_summary.record_overwritten_props(ctx);
        }
    }

    /// A call to the class's own method leaves `$this`'s properties as that method stores them.
    fn store_own_call_props(
        &mut self,
        target: &php_ast::owned::Expr,
        method: &php_ast::owned::Expr,
        args: &[php_ast::owned::Arg],
        ctx: &mut FlowState,
    ) {
        for (prop, taint) in
            crate::taint::own_call_prop_writes(target, method, args, ctx, self.db, &self.file)
        {
            let held = ctx.prop_taint("this", prop.as_str());
            ctx.set_prop_taint("this", prop.as_str(), held | taint);
            self.taint_summary.note_prop_overwrite(prop);
        }
    }

    /// A call to a method that stores its arguments in a static property taints that property.
    fn store_static_call_props(
        &mut self,
        target: &php_ast::owned::Expr,
        method: &php_ast::owned::Expr,
        args: &[php_ast::owned::Arg],
        ctx: &mut FlowState,
    ) {
        let class = match &target.kind {
            ExprKind::Name(_) => {
                let name = target
                    .kind
                    .resolve_name_ref(self.db, &self.file)
                    .unwrap_or_default();
                match name.as_str() {
                    "self" | "static" => ctx.self_fqcn.clone(),
                    "parent" => ctx.parent_fqcn.clone(),
                    resolved => Some(Arc::from(resolved)),
                }
            }
            ExprKind::Variable(var) if &**var == "this" => ctx.self_fqcn.clone(),
            ExprKind::Variable(var) => match ctx.get_var(var).types.as_slice() {
                [mir_types::Atomic::TNamedObject { fqcn, .. }] => Some(Arc::from(fqcn.as_ref())),
                _ => None,
            },
            _ => None,
        };
        let Some(class) = class else { return };
        let receiver = matches!(&target.kind, ExprKind::Variable(_)).then_some(target);
        let writes = crate::taint::static_call_prop_writes(
            &class, method, args, receiver, ctx, self.db, &self.file,
        );
        for (classes, prop, taint) in writes {
            for class in &classes {
                ctx.taint_static_prop(class, prop.as_str(), taint);
            }
            if ctx.self_fqcn.as_deref() == Some(class.as_ref()) {
                self.taint_summary.record_prop_write(
                    crate::db::taint_sources::prop_key(prop.as_str(), true),
                    taint,
                );
            }
        }
    }

    /// Storing data in an `ArrayObject` or `ArrayIterator` taints the container.
    fn taint_array_container(
        &mut self,
        target: &php_ast::owned::Expr,
        method: &php_ast::owned::Expr,
        args: &[php_ast::owned::Arg],
        ctx: &mut FlowState,
    ) {
        let (ExprKind::Variable(var), ExprKind::Identifier(name)) = (&target.kind, &method.kind)
        else {
            return;
        };
        if !["append", "exchangearray", "offsetset"]
            .iter()
            .any(|stores| name.eq_ignore_ascii_case(stores))
        {
            return;
        }
        let taint = crate::taint::args_taint(args, ctx, self.db, &self.file);
        if taint.is_empty() {
            return;
        }
        let is_container = ctx.get_var(var).types.iter().any(|atom| {
            matches!(atom, mir_types::Atomic::TNamedObject { fqcn, .. }
                if ["ArrayObject", "ArrayIterator"]
                    .iter()
                    .any(|class| crate::db::extends_or_implements(self.db, fqcn.as_ref(), class)))
        });
        if is_container {
            ctx.taint_var(var, taint);
        }
    }

    /// A call on a local object leaves its properties as the method stores them.
    fn store_receiver_call_props(
        &mut self,
        target: &php_ast::owned::Expr,
        method: &php_ast::owned::Expr,
        args: &[php_ast::owned::Arg],
        ctx: &mut FlowState,
    ) {
        let ExprKind::Variable(var) = &target.kind else {
            return;
        };
        let var = var.trim_start_matches('$');
        if var == "this" {
            return;
        }
        for (prop, taint) in
            crate::taint::receiver_call_prop_writes(var, method, args, ctx, self.db, &self.file)
        {
            ctx.set_prop_taint_through_aliases(var, prop.as_str(), taint);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: &'a dyn MirDatabase,
        file: Arc<str>,
        source: &'a str,
        source_map: &'a php_rs_parser::source_map::SourceMap,
        issues: &'a mut IssueBuffer,
        symbols: Option<&'a mut Vec<ResolvedSymbol>>,
        navigation_facts: &'a std::cell::RefCell<Vec<NavigationFact>>,
        resolved_navigation_facts: &'a std::cell::RefCell<Vec<ResolvedNavigationFact>>,
        php_version: PhpVersion,
        mode: AnalysisMode,
        yielded_types: &'a mut Vec<(Type, Type)>,
        taint_summary: &'a mut crate::taint::TaintSummary,
        reference_key_cache: &'a mut ReferenceKeyCache,
    ) -> Self {
        Self {
            db,
            file,
            source,
            source_map,
            issues,
            symbols,
            navigation_facts,
            resolved_navigation_facts,
            php_version,
            mode,
            strict_types: false,
            collect_symbols: true,
            capture_symbol_types: true,
            codebase_symbols_only: false,
            record_reference_locations: true,
            collect_navigation_facts: false,
            collect_resolved_navigation_facts: false,
            in_existence_check: false,
            vivifying_reference: false,
            callback_param_hints: None,
            in_array_access_base: false,
            yielded_types,
            taint_summary,
            plugins: mir_plugin::snapshot(),
            reference_key_cache,
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

    /// `define('NAME', …)`: the quoted name is the constant's declaration site.
    fn record_define_name_fact(&self, fc: &php_ast::owned::FunctionCallExpr) {
        let Some(callee) = fc.name.kind.name_bare() else {
            return;
        };
        if !callee.eq_ignore_ascii_case("define") {
            return;
        }
        let Some(name_expr) = fc.args.first().and_then(|a| a.value()) else {
            return;
        };
        let ExprKind::String(name) = &name_expr.kind else {
            return;
        };
        let span = name_expr.span;
        // Escapes make the source longer than the value; skip those.
        if name.is_empty() || (span.end - span.start) as usize != name.len() + 2 {
            return;
        }
        self.navigation_facts.borrow_mut().push(NavigationFact {
            span: php_ast::Span::new(span.start + 1, span.end - 1),
            expr_span: None,
            name: crate::Name::global_constant(name.as_str()),
        });
    }

    /// Record a resolved symbol.
    pub fn record_symbol(&mut self, span: php_ast::Span, kind: ReferenceKind, resolved_type: Type) {
        let name = kind.to_name();
        if self.codebase_symbols_only && name.is_none() {
            return;
        }
        if self.collect_navigation_facts {
            if let Some(name) = name.clone() {
                self.navigation_facts.borrow_mut().push(NavigationFact {
                    span,
                    expr_span: None,
                    name,
                });
            }
        }
        if self.collect_resolved_navigation_facts {
            self.resolved_navigation_facts
                .borrow_mut()
                .push(ResolvedNavigationFact {
                    span,
                    expr_span: None,
                    kind: kind.clone(),
                    resolved_type: self.symbol_type(resolved_type.clone()),
                });
        }
        if !self.collect_symbols {
            return;
        }
        let resolved_type = self.symbol_type(resolved_type);
        if let Some(symbols) = self.symbols.as_deref_mut() {
            symbols.push(ResolvedSymbol {
                file: self.file.clone(),
                span,
                expr_span: None,
                kind,
                resolved_type,
            });
        }
    }

    pub fn record_symbol_with_expr_span(
        &mut self,
        span: php_ast::Span,
        expr_span: php_ast::Span,
        kind: ReferenceKind,
        resolved_type: Type,
    ) {
        let name = kind.to_name();
        if self.codebase_symbols_only && name.is_none() {
            return;
        }
        if self.collect_navigation_facts {
            if let Some(name) = name.clone() {
                self.navigation_facts.borrow_mut().push(NavigationFact {
                    span,
                    expr_span: Some(expr_span),
                    name,
                });
            }
        }
        if self.collect_resolved_navigation_facts {
            self.resolved_navigation_facts
                .borrow_mut()
                .push(ResolvedNavigationFact {
                    span,
                    expr_span: Some(expr_span),
                    kind: kind.clone(),
                    resolved_type: self.symbol_type(resolved_type.clone()),
                });
        }
        if !self.collect_symbols {
            return;
        }
        let resolved_type = self.symbol_type(resolved_type);
        if let Some(symbols) = self.symbols.as_deref_mut() {
            symbols.push(ResolvedSymbol {
                file: self.file.clone(),
                span,
                expr_span: Some(expr_span),
                kind,
                resolved_type,
            });
        }
    }

    fn symbol_type(&self, resolved_type: Type) -> Type {
        if self.capture_symbol_types {
            resolved_type
        } else {
            Type::mixed()
        }
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

    /// `analyze` for a call argument: `Enum::Case` keeps its case so template inference binds the
    /// case, not the whole enum.
    pub(crate) fn analyze_arg(&mut self, expr: &php_ast::owned::Expr, ctx: &mut FlowState) -> Type {
        let ty = self.analyze(expr, ctx);
        let ExprKind::ClassConstAccess(cca) = &expr.kind else {
            return ty;
        };
        let (ExprKind::Identifier(case), [Atomic::TNamedObject { fqcn, type_params }]) =
            (&cca.member.kind, ty.types.as_slice())
        else {
            return ty;
        };
        if !type_params.is_empty() {
            return ty;
        }
        let is_case = matches!(
            crate::db::find_class_like(self.db, crate::db::Fqcn::from_str(self.db, fqcn)),
            Some(crate::db::ClassLike::Enum(e)) if e.cases.contains_key(case.as_ref())
        );
        if !is_case {
            return ty;
        }
        Type::single(Atomic::TLiteralEnumCase {
            enum_fqcn: *fqcn,
            case_name: case.as_ref().into(),
        })
    }

    pub fn analyze(&mut self, expr: &php_ast::owned::Expr, ctx: &mut FlowState) -> Type {
        let ty = crate::recursion::ensure_stack(|| self.analyze_inner(expr, ctx));
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
                let mut any_non_empty_literal = false;
                for part in parts.iter() {
                    match part {
                        php_ast::owned::StringPart::Literal(s) => {
                            if !s.is_empty() {
                                any_non_empty_literal = true;
                            }
                            literal.push_str(s.as_str());
                        }
                        php_ast::owned::StringPart::Expr(e) => {
                            all_literal = false;
                            let expr_ty = self.analyze(e, ctx);
                            self.check_interpolation_implicit_to_string_cast(&expr_ty, e.span);
                        }
                        _ => {}
                    }
                }
                if all_literal {
                    Type::single(Atomic::TLiteralString(literal.into()))
                } else if any_non_empty_literal {
                    // A non-empty literal part stays in the result no matter how
                    // the embedded expressions coerce to string, so the whole
                    // string is guaranteed non-empty — same rule as `.`
                    // concatenation (`is_non_empty_when_concat` in binary.rs).
                    Type::single(Atomic::TNonEmptyString)
                } else {
                    Type::single(Atomic::TString)
                }
            }
            ExprKind::Nowdoc { value, .. } => {
                Type::single(Atomic::TLiteralString(value.as_ref().into()))
            }
            ExprKind::ShellExec(parts) => {
                // The backtick operator is a shell-exec sink exactly like
                // `shell_exec()`/`exec()`, which already run their argument
                // through `reaches_sink` -> `TaintedShell` — this arm used
                // to discard `parts` entirely (`_`), so an interpolated
                // tainted value never got analyzed OR taint-checked here.
                for part in parts.iter() {
                    if let php_ast::owned::StringPart::Expr(e) = part {
                        let expr_ty = self.analyze(e, ctx);
                        self.check_interpolation_implicit_to_string_cast(&expr_ty, e.span);
                    }
                }
                if crate::taint::reaches_sink(
                    expr,
                    crate::taint::Taint::SHELL,
                    &IssueKind::TaintedShell,
                    ctx,
                    self.db,
                    &self.file,
                    self.taint_summary,
                ) {
                    self.emit_taint(
                        IssueKind::TaintedShell,
                        expr.span,
                        expr,
                        crate::taint::Taint::SHELL,
                        ctx,
                    );
                }
                if self.db.is_forbidden_function("shell_exec") {
                    self.emit(
                        IssueKind::ForbiddenCode {
                            message: "Use of shell_exec (backtick) is forbidden".to_string(),
                        },
                        Severity::Warning,
                        expr.span,
                    );
                }
                Type::single(Atomic::TString)
            }

            // --- Variables --------------------------------------------------
            ExprKind::Variable(name) => self.analyze_variable(name.as_ref(), expr, ctx),
            ExprKind::VariableVariable(inner) => self.analyze_variable_variable(inner, ctx),
            ExprKind::Name(name) if name.kind != php_ast::ast::NameKind::Error => {
                self.analyze_constant(name, expr, ctx)
            }

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
                if crate::taint::reaches_sink(
                    inner,
                    crate::taint::Taint::HTML,
                    &IssueKind::TaintedHtml,
                    ctx,
                    self.db,
                    &self.file,
                    self.taint_summary,
                ) {
                    self.emit_taint(
                        IssueKind::TaintedHtml,
                        expr.span,
                        inner,
                        crate::taint::Taint::HTML,
                        ctx,
                    );
                }
                // @pure implies no side effects at all — same purity
                // violation `echo` checks (stmt/expressions.rs), for the
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
            ExprKind::New(n) => {
                let ty = self.analyze_new(n, expr.span, ctx);
                ctx.invalidate_escaped_vars();
                ty
            }

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
                    anon_supers.push(crate::parser::name_to_string_owned(parent).into_owned());
                }
                for iface in anon.implements.iter() {
                    anon_supers.push(crate::parser::name_to_string_owned(iface).into_owned());
                }
                for name in anon_supers {
                    let resolved = crate::db::resolve_name(self.db, self.file.as_ref(), &name);
                    let lc = resolved.trim_start_matches('\\').to_ascii_lowercase();
                    self.record_impl_ref(&lc, kw_span);
                }
                let mut sa = crate::stmt::StatementsAnalyzer::new(
                    self.db,
                    self.file.clone(),
                    self.source,
                    self.source_map,
                    self.issues,
                    self.symbols.as_deref_mut(),
                    self.navigation_facts,
                    self.resolved_navigation_facts,
                    self.php_version,
                    self.mode,
                );
                sa.collect_symbols = self.collect_symbols;
                sa.capture_symbol_types = self.capture_symbol_types;
                sa.codebase_symbols_only = self.codebase_symbols_only;
                sa.record_reference_locations = self.record_reference_locations;
                sa.collect_navigation_facts = self.collect_navigation_facts;
                sa.collect_resolved_navigation_facts = self.collect_resolved_navigation_facts;
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
                if let Some(resolved) = class.kind.resolve_name_ref(self.db, self.file.as_ref()) {
                    let fqcn = match ctx.self_fqcn.as_deref() {
                        Some(self_fqcn) if resolved.eq_ignore_ascii_case("self") => {
                            self_fqcn.to_string()
                        }
                        _ => resolved,
                    };
                    if !matches!(fqcn.as_str(), "self" | "static" | "parent") {
                        self.record_dynamic_member_ref(fqcn, member.span);
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
                let ty =
                    crate::call::CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, false);
                ctx.invalidate_escaped_vars();
                self.expose_props_to_call(&mc.object, &mc.args, ctx);
                self.store_own_call_props(&mc.object, &mc.method, &mc.args, ctx);
                self.store_receiver_call_props(&mc.object, &mc.method, &mc.args, ctx);
                self.store_static_call_props(&mc.object, &mc.method, &mc.args, ctx);
                self.taint_array_container(&mc.object, &mc.method, &mc.args, ctx);
                partial_application_type(&mc.args, ty, || {
                    self.method_call_closure(&mc.object, &mc.method, ctx)
                })
            }

            ExprKind::NullsafeMethodCall(mc) => {
                let ty =
                    crate::call::CallAnalyzer::analyze_method_call(self, mc, ctx, expr.span, true);
                ctx.invalidate_escaped_vars();
                self.expose_props_to_call(&mc.object, &mc.args, ctx);
                if is_partial_application(&mc.args) {
                    Type::nullable(partial_closure(&mc.args, || {
                        self.method_call_closure(&mc.object, &mc.method, ctx)
                    }))
                } else {
                    ty
                }
            }

            ExprKind::StaticMethodCall(smc) => {
                let ty = crate::call::CallAnalyzer::analyze_static_method_call(
                    self, smc, ctx, expr.span,
                );
                ctx.invalidate_escaped_vars();
                self.expose_props_to_call(&smc.class, &smc.args, ctx);
                self.store_own_call_props(&smc.class, &smc.method, &smc.args, ctx);
                self.store_static_call_props(&smc.class, &smc.method, &smc.args, ctx);
                partial_application_type(&smc.args, ty, || {
                    self.static_call_closure(&smc.class, &smc.method, ctx)
                })
            }

            ExprKind::StaticMethodCallDynamic(smc) => {
                crate::taint::check_callable_target(self, &smc.method, ctx, expr.span);
                if !smc.class.kind.is_name_ref() {
                    crate::taint::check_callable_target(self, &smc.class, ctx, expr.span);
                }
                let ty = crate::call::CallAnalyzer::analyze_static_dyn_method_call(self, smc, ctx);
                ctx.invalidate_escaped_vars();
                ty
            }

            // --- Function calls --------------------------------------------
            ExprKind::FunctionCall(fc) => {
                if self.collect_navigation_facts {
                    self.record_define_name_fact(fc);
                }
                let ty = crate::call::CallAnalyzer::analyze_function_call(self, fc, ctx, expr.span);
                ctx.invalidate_escaped_vars();
                partial_application_type(&fc.args, ty, || self.function_call_closure(fc, ctx))
            }

            // --- Closures / arrow functions --------------------------------
            ExprKind::Closure(c) => self.analyze_closure(c, expr.span, ctx),

            ExprKind::ArrowFunction(af) => self.analyze_arrow_function(af, expr.span, ctx),

            ExprKind::CallableCreate(cc) => self.callable_create_type(cc, expr.span, ctx),

            // --- Match expression ------------------------------------------
            ExprKind::Match(m) => self.analyze_match(m, expr.span, ctx),

            // --- Throw as expression (PHP 8) --------------------------------
            ExprKind::Throw(e) => {
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
                // A tainted path is local/remote file inclusion — the same
                // File-sink risk as fopen()/file_get_contents(), but this is
                // its own AST node (not a FunctionCall), so classify_sink's
                // dispatch never sees it.
                if crate::taint::reaches_sink(
                    inner,
                    crate::taint::Taint::FILE,
                    &IssueKind::TaintedInput {
                        sink: "include".to_string(),
                    },
                    ctx,
                    self.db,
                    &self.file,
                    self.taint_summary,
                ) {
                    self.emit_taint(
                        IssueKind::TaintedInput {
                            sink: "include".to_string(),
                        },
                        expr.span,
                        inner,
                        crate::taint::Taint::FILE,
                        ctx,
                    );
                }
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
                // `eval()` is its own AST node (not a FunctionCall), so
                // classify_sink's name-based dispatch never sees it — tainted
                // code reaching it is arbitrary code execution, the most
                // dangerous sink this analyzer models.
                if crate::taint::reaches_sink(
                    inner,
                    crate::taint::Taint::EVAL,
                    &IssueKind::TaintedInput {
                        sink: "eval".to_string(),
                    },
                    ctx,
                    self.db,
                    &self.file,
                    self.taint_summary,
                ) {
                    self.emit_taint(
                        IssueKind::TaintedInput {
                            sink: "eval".to_string(),
                        },
                        expr.span,
                        inner,
                        crate::taint::Taint::EVAL,
                        ctx,
                    );
                }
                Type::mixed()
            }

            // --- Exit -------------------------------------------------------
            ExprKind::Exit(opt) => {
                if let Some(e) = opt {
                    let ty = self.analyze(e, ctx);
                    // A string operand is printed; an int operand is the exit status.
                    let is_status = ty
                        .types
                        .iter()
                        .all(|a| matches!(a, Atomic::TInt | Atomic::TLiteralInt(_)));
                    if !is_status
                        && crate::taint::reaches_sink(
                            e,
                            crate::taint::Taint::HTML,
                            &IssueKind::TaintedHtml,
                            ctx,
                            self.db,
                            &self.file,
                            self.taint_summary,
                        )
                    {
                        self.emit_taint(
                            IssueKind::TaintedHtml,
                            expr.span,
                            e,
                            crate::taint::Taint::HTML,
                            ctx,
                        );
                    }
                }
                ctx.diverges = true;
                Type::single(Atomic::TNever)
            }

            // --- Error node (parse error placeholder) ----------------------
            ExprKind::Error => Type::mixed(),

            // --- Omitted array slot (e.g. [, $b] destructuring) ------------
            ExprKind::Omit => Type::single(Atomic::TNull),
            _ => Type::mixed(),
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
                let found = name_expr
                    .kind
                    .function_ref(self.db, &self.file)
                    .and_then(|name| {
                        name.find(self.db, &self.file)
                            .map(|(resolved, f)| (resolved.to_string(), f))
                    });
                if let Some((resolved_fqn, f)) = found {
                    self.record_function_ref(&f.fqn, name_expr.span);
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
                    // No receiver for a plain function — self/static/parent can't
                    // legally appear in its signature, so an empty fqcn/type-param
                    // list is a safe no-op for build_closure_from_resolved_params's
                    // static-substitution step. Routed through the same helper as
                    // the method FCC cases below so the function's own @template
                    // params get bound to their declared bound (see the helper's
                    // doc comment) instead of leaking as bare, unchecked
                    // TTemplateParam atoms into every call through the closure.
                    let empty_fqcn: Arc<str> = Arc::from("");
                    let closure_ty = Type::single(Self::build_closure_from_resolved_params(
                        self.db,
                        &f.params,
                        return_ty,
                        &rustc_hash::FxHashMap::default(),
                        &f.template_params,
                        &empty_fqcn,
                        &[],
                        f.is_pure,
                    ));
                    // Hover on the name shows the Closure the expression produces.
                    self.record_symbol(
                        name_expr.span,
                        ReferenceKind::FunctionCall(f.fqn.clone()),
                        closure_ty.clone(),
                    );
                    return closure_ty;
                }
                if let Some(fn_name) = name_expr.kind.name_text() {
                    let resolved = name_expr
                        .kind
                        .function_ref(self.db, &self.file)
                        .map(|r| r.qualified.into_owned())
                        .unwrap_or_default();
                    crate::call::report_undefined_function(
                        self, &fn_name, &resolved, ctx, outer_span, false,
                    );
                }
                Type::single(bare_closure())
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
                        return Type::single(bare_closure());
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
                    // `fqcn` is already a canonical, resolved receiver type (from
                    // `named_object_fqcn()`) — re-running `resolve_name`'s
                    // raw-source-text rules on it, rather than trusting it as-is,
                    // mis-resolves a `\GlobalClass` receiver in a namespaced file;
                    // `db::resolve_receiver_fqcn` handles exactly this.
                    let fqcn_resolved =
                        crate::db::resolve_receiver_fqcn(self.db, self.file.as_ref(), fqcn);
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
                        let closure = Self::resolved_method_closure(
                            self.db,
                            &fqcn_arc,
                            &receiver_type_params,
                            resolved,
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
                Type::single(bare_closure())
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
                        if let Some(resolved) =
                            class.kind.resolve_name_ref(self.db, self.file.as_ref())
                        {
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
                                self.record_dynamic_member_ref(fqcn, method.span);
                            }
                        } else {
                            let class_ty = self.analyze(class, ctx);
                            self.record_dynamic_member_access(&class_ty, method.span);
                        }
                        self.analyze(method, ctx);
                        return Type::single(bare_closure());
                    }
                };
                let method_name_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                let mut receiver_type_params: Vec<Type> = Vec::new();
                let is_named_class = class.kind.is_name_ref();
                let fqcn = match class.kind.resolve_name_ref(self.db, self.file.as_ref()) {
                    Some(resolved) => match crate::util::php_ident_lowercase(&resolved).as_str() {
                        "self" => ctx.self_fqcn.as_deref().unwrap_or("self").to_string(),
                        "parent" => ctx.parent_fqcn.as_deref().unwrap_or("parent").to_string(),
                        "static" => ctx
                            .static_fqcn
                            .as_deref()
                            .unwrap_or(ctx.self_fqcn.as_deref().unwrap_or("static"))
                            .to_string(),
                        _ => resolved,
                    },
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
                            None => return Type::single(bare_closure()),
                        }
                    }
                };
                // A named class token (`Foo::bar(...)`) must resolve to a real
                // class — unlike the object-derived branch above, whose fqcn
                // already came from a resolved (thus existing) receiver type.
                // Without this check an undefined class here silently produced
                // no diagnostic at all, unlike the identical `Foo::bar()`
                // direct-call form.
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
                    return Type::single(bare_closure());
                }
                let fqcn_arc: Arc<str> = Arc::from(fqcn.as_str());
                if is_named_class {
                    self.record_class_ref(fqcn_arc.as_ref(), class.span);
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
                    return Type::single(Self::resolved_method_closure(
                        self.db,
                        &fqcn_arc,
                        &receiver_type_params,
                        resolved,
                    ));
                }
                self.emit_undefined_static_method_for_callable(
                    &fqcn_arc,
                    &method_name,
                    method.span,
                );
                Type::single(bare_closure())
            }
            _ => Type::single(bare_closure()),
        }
    }

    /// UndefinedMethod check for a first-class-callable (`$obj->method(...)`)
    /// whose method didn't resolve — mirrors call/method.rs's ordinary-call
    /// suppression rules (interface/abstract/trait receivers, `__call`, and an
    /// active `method_exists()` guard).
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
        let guarded_by_method_exists = ctx.is_method_guarded(
            crate::narrowing::extract_expr_guard_key(object, ctx, self.db, &self.file),
            fqcn,
            method_name_lower,
        );
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

    /// The `Closure` a first-class callable of this function call would produce.
    fn function_call_closure(
        &self,
        fc: &php_ast::owned::FunctionCallExpr,
        ctx: &FlowState,
    ) -> Option<Atomic> {
        if let ExprKind::Variable(var) = &fc.name.kind {
            return match ctx.get_var(var).types.as_slice() {
                [closure @ Atomic::TClosure { .. }] => Some(closure.clone()),
                _ => None,
            };
        }
        let (_, f) = fc
            .name
            .kind
            .function_ref(self.db, &self.file)?
            .find(self.db, &self.file)?;
        let return_ty = f
            .return_type
            .as_deref()
            .cloned()
            .unwrap_or_else(Type::mixed);
        Some(Self::build_closure_from_resolved_params(
            self.db,
            &f.params,
            return_ty,
            &rustc_hash::FxHashMap::default(),
            &f.template_params,
            &Arc::from(""),
            &[],
            f.is_pure,
        ))
    }

    /// The `Closure` of `$var->method(...)`; other receivers are not re-analyzed.
    fn method_call_closure(
        &self,
        object: &php_ast::owned::Expr,
        method: &php_ast::owned::Expr,
        ctx: &FlowState,
    ) -> Option<Atomic> {
        let (ExprKind::Variable(var), ExprKind::Identifier(method_name)) =
            (&object.kind, &method.kind)
        else {
            return None;
        };
        let (fqcn, receiver_type_params) =
            ctx.get_var(var)
                .remove_null()
                .types
                .iter()
                .find_map(|a| match a {
                    Atomic::TNamedObject { fqcn, type_params } => {
                        Some((*fqcn, type_params.to_vec()))
                    }
                    _ => None,
                })?;
        let fqcn: Arc<str> = Arc::from(
            crate::db::resolve_receiver_fqcn(self.db, self.file.as_ref(), fqcn.as_ref()).as_str(),
        );
        let resolved = crate::call::method::resolve_method_from_db(
            self.db,
            &fqcn,
            &crate::util::php_ident_lowercase(method_name.as_ref()),
        )?;
        Some(Self::resolved_method_closure(
            self.db,
            &fqcn,
            &receiver_type_params,
            resolved,
        ))
    }

    /// The `Closure` of `Class::method(...)` for a named class.
    fn static_call_closure(
        &self,
        class: &php_ast::owned::Expr,
        method: &php_ast::owned::Expr,
        ctx: &FlowState,
    ) -> Option<Atomic> {
        let ExprKind::Identifier(method_name) = &method.kind else {
            return None;
        };
        let resolved_class = class.kind.resolve_name_ref(self.db, self.file.as_ref())?;
        let fqcn: Arc<str> = match crate::util::php_ident_lowercase(&resolved_class).as_str() {
            "self" => ctx.self_fqcn.clone()?,
            "parent" => ctx.parent_fqcn.clone()?,
            "static" => ctx.static_fqcn.clone().or_else(|| ctx.self_fqcn.clone())?,
            _ => Arc::from(resolved_class.as_str()),
        };
        let resolved = crate::call::method::resolve_method_from_db(
            self.db,
            &fqcn,
            &crate::util::php_ident_lowercase(method_name.as_ref()),
        )?;
        Some(Self::resolved_method_closure(self.db, &fqcn, &[], resolved))
    }

    /// The `Closure` of `resolved` called on `fqcn`, with the receiver's template bindings applied.
    fn resolved_method_closure(
        db: &dyn crate::db::MirDatabase,
        fqcn: &Arc<str>,
        receiver_type_params: &[Type],
        resolved: crate::call::method::ResolvedMethod,
    ) -> Atomic {
        let class_tps = crate::db::class_template_params(db, fqcn)
            .map(|tps| tps.to_vec())
            .unwrap_or_default();
        let mut bindings = crate::generic::build_class_bindings(&class_tps, receiver_type_params);
        let inherited_bindings = crate::db::inherited_template_bindings(db, fqcn, &bindings);
        // Own bindings win only when the receiver's class declares the method; otherwise the
        // declaring ancestor's do (same collision guard as call/method.rs).
        if resolved.owner_fqcn.as_ref() == fqcn.as_ref() {
            for (k, v) in inherited_bindings {
                bindings.entry(k).or_insert(v);
            }
        } else {
            bindings.extend(inherited_bindings);
        }
        Self::build_closure_from_resolved_params(
            db,
            &resolved.params,
            resolved.return_ty_raw,
            &bindings,
            &resolved.template_params,
            fqcn,
            receiver_type_params,
            resolved.is_pure,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn build_closure_from_resolved_params(
        db: &dyn crate::db::MirDatabase,
        params: &[mir_codebase::definitions::DeclaredParam],
        return_ty: Type,
        bindings: &rustc_hash::FxHashMap<mir_types::Name, Type>,
        own_template_params: &[mir_codebase::definitions::TemplateParam],
        receiver_fqcn: &Arc<str>,
        receiver_type_params: &[Type],
        is_pure: bool,
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
            crate::call::substitute_static_in_return(db, t, receiver_fqcn, receiver_type_params)
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
                is_pure,
                is_static: false,
            }),
        }
    }

    /// Record a reference location for `symbol_key` at `span`, unless in inference-only mode.
    pub(crate) fn record_ref(&self, symbol_key: Arc<str>, span: php_ast::Span) {
        if self.mode == AnalysisMode::InferenceOnly || !self.record_reference_locations {
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

    pub(crate) fn record_class_ref(&mut self, fqcn: impl AsRef<str>, span: php_ast::Span) {
        let key = self.reference_key_cache.class(fqcn.as_ref());
        self.record_ref(key, span);
    }

    pub(crate) fn record_function_ref(&mut self, fqn: impl AsRef<str>, span: php_ast::Span) {
        let fqn = fqn.as_ref();
        let key = Arc::<str>::from(format!("fn:{fqn}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_method_ref(
        &mut self,
        class: impl AsRef<str>,
        name: impl AsRef<str>,
        span: php_ast::Span,
    ) {
        let class = class.as_ref();
        let name = name.as_ref();
        let key = Arc::<str>::from(format!("meth:{class}::{name}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_property_ref(
        &mut self,
        class: impl AsRef<str>,
        name: impl AsRef<str>,
        span: php_ast::Span,
    ) {
        let class = class.as_ref();
        let name = name.as_ref();
        let key = Arc::<str>::from(format!("prop:{class}::{name}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_global_constant_ref(&mut self, fqn: impl AsRef<str>, span: php_ast::Span) {
        let fqn = fqn.as_ref();
        let key = Arc::<str>::from(format!("gcnst:{fqn}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_dynamic_member_ref(&mut self, fqcn: impl AsRef<str>, span: php_ast::Span) {
        let fqcn = fqcn.as_ref();
        let key = Arc::<str>::from(format!("dyn:{fqcn}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_impl_ref(&mut self, fqcn: impl AsRef<str>, span: php_ast::Span) {
        let fqcn = fqcn.as_ref();
        let key = Arc::<str>::from(format!("impl:{fqcn}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_propname_ref(&mut self, name: impl AsRef<str>, span: php_ast::Span) {
        let name = name.as_ref();
        let key = Arc::<str>::from(format!("propname:{name}"));
        self.record_ref(key, span);
    }

    pub(crate) fn record_trait_use_property_ref(
        &mut self,
        class: impl AsRef<str>,
        name: impl AsRef<str>,
        span: php_ast::Span,
    ) {
        let class = class.as_ref();
        let name = name.as_ref();
        let key = Arc::<str>::from(format!("traituse:{class}::{name}"));
        self.record_ref(key, span);
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
    pub(crate) fn record_dynamic_member_access(&mut self, obj_ty: &Type, span: php_ast::Span) {
        if self.mode == AnalysisMode::InferenceOnly {
            return;
        }
        for atomic in obj_ty.remove_null().types.iter() {
            let fqcn = atomic.named_object_fqcn().or_else(|| match atomic {
                Atomic::TClassString(Some(fqcn)) => Some(fqcn.as_ref()),
                _ => None,
            });
            if let Some(fqcn) = fqcn {
                self.record_dynamic_member_ref(fqcn, span);
            }
        }
    }

    /// Walk a type hint and emit `UndefinedClass` for any named type not in the codebase.
    fn check_type_hint(&mut self, hint: &php_ast::owned::TypeHint) {
        use php_ast::owned::TypeHintKind;
        match &hint.kind {
            TypeHintKind::Named(name) => {
                let name_str = crate::parser::name_to_string_owned(name);
                if crate::util::is_native_type_name(&name_str) {
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
                    self.record_class_ref(fqcn.as_ref(), hint.span);
                    self.record_symbol(
                        hint.span,
                        ReferenceKind::ClassReference(fqcn.clone()),
                        crate::symbol::class_token_type(&fqcn),
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
            _ => {}
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
        calling_class: Option<&str>,
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
                calling_class,
                issues: Default::default(),
            };
            let provided = plugins.function_return_type(&event);
            let issues = event.issues.take();
            if let Some(provided) = provided {
                *return_ty = self.resolve_provided_type(provided);
            }
            self.emit_plugin_issues(issues, span);
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
        calling_class: Option<&str>,
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
                calling_class,
                issues: Default::default(),
            };
            let provided = plugins.method_return_type(&normalized, &event);
            let issues = event.issues.take();
            self.emit_plugin_issues(issues, span);
            if let Some(provided) = provided {
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
        let issue = self.build_issue(kind, severity, span);
        self.issues.add(issue);
    }

    /// Location of `span` in the file under analysis.
    pub(crate) fn span_location(&self, span: php_ast::Span) -> Location {
        let (line, col_start) = self.offset_to_line_col(span.start);
        let (line_end, col_end) = if span.start < span.end {
            self.offset_to_line_col(span.end)
        } else {
            (line, col_start)
        };
        Location {
            file: self.file.clone(),
            line,
            line_end,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        }
    }

    pub(crate) fn build_issue(
        &self,
        kind: IssueKind,
        severity: Severity,
        span: php_ast::Span,
    ) -> Issue {
        let mut issue = Issue::new(kind, self.span_location(span));
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
        issue
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
        // Expands docblock `Class::*` refs into their constant types.
        let ty = crate::subtype::canonical_enum_cases(self.db, ty);
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
        // lines. Clamping it to `col_start + 1` would point past the end
        // line's actual content (e.g. `TooManyArguments` on 0-arg `new` calls).
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
