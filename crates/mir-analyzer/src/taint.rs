/// Taint analysis helpers.
///
/// A value is "tainted" when it originates from user-controlled input
/// (superglobals, request headers, `@taint-source` calls). Taint is tracked
/// per [`Taint`] kind and propagates through assignments, string operations,
/// array access and pass-through builtins; a sanitizer removes only the kinds
/// it neutralizes.
///
/// When data tainted for a sink's kind reaches it, the matching `Tainted*`
/// issue is emitted.
use php_ast::owned::{Expr, ExprKind, StringPart};

use crate::flow_state::FlowState;

/// Set of sink kinds a value is still dangerous for.
///
/// The low [`KIND_BITS`] hold taint from real sources. While a function body
/// is analyzed for its [`TaintSummary`], each of the first
/// [`MAX_TRACKED_PARAMS`] parameters gets its own slot of the same width, so a
/// sanitizer applied to a parameter is remembered per parameter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Taint(u128);

const KIND_BITS: u32 = 14;
const KIND_MASK: u128 = (1 << KIND_BITS) - 1;
pub const MAX_TRACKED_PARAMS: usize = 8;

/// `kinds` copied into the source slot and every parameter slot.
const fn replicate(kinds: u128) -> u128 {
    let kinds = kinds & KIND_MASK;
    let mut out = 0;
    let mut slot = 0;
    while slot <= MAX_TRACKED_PARAMS as u32 {
        out |= kinds << (slot * KIND_BITS);
        slot += 1;
    }
    out
}

impl Taint {
    pub const NONE: Taint = Taint(0);
    pub const HTML: Taint = Taint(1 << 0);
    pub const SQL: Taint = Taint(1 << 1);
    pub const SHELL: Taint = Taint(1 << 2);
    pub const HEADER: Taint = Taint(1 << 3);
    pub const COOKIE: Taint = Taint(1 << 4);
    pub const FILE: Taint = Taint(1 << 5);
    pub const UNSERIALIZE: Taint = Taint(1 << 6);
    pub const EVAL: Taint = Taint(1 << 7);
    pub const LLM_PROMPT: Taint = Taint(1 << 8);
    pub const LDAP: Taint = Taint(1 << 9);
    pub const XPATH: Taint = Taint(1 << 10);
    /// Any `@taint-sink` kind without a dedicated bit.
    pub const OTHER: Taint = Taint(1 << 11);
    pub const SSRF: Taint = Taint(1 << 12);
    pub const CALLABLE: Taint = Taint(1 << 13);
    pub const ALL: Taint = Taint(KIND_MASK);

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn intersects(self, other: Taint) -> bool {
        self.0 & other.0 != 0
    }

    /// Remove `kinds` from the source slot and every parameter slot.
    pub const fn without(self, kinds: Taint) -> Taint {
        Taint(self.0 & !replicate(kinds.0))
    }

    /// Keep only `kinds` in every slot.
    pub const fn restrict_to(self, kinds: Taint) -> Taint {
        Taint(self.0 & replicate(kinds.0))
    }

    /// Add `kinds` to every slot that is already tainted.
    pub fn restore(self, kinds: Taint) -> Taint {
        let mut out = self.0;
        for slot in 0..=MAX_TRACKED_PARAMS as u32 {
            if (self.0 >> (slot * KIND_BITS)) & KIND_MASK != 0 {
                out |= (kinds.0 & KIND_MASK) << (slot * KIND_BITS);
            }
        }
        Taint(out)
    }

    /// Parameter labels only.
    pub const fn labels(self) -> Taint {
        Taint(self.0 & !KIND_MASK)
    }

    /// Taint from real sources only.
    pub const fn concrete(self) -> Taint {
        Taint(self.0 & KIND_MASK)
    }

    /// The label seeded into parameter `index` for summary analysis.
    pub const fn param(index: usize) -> Taint {
        Taint(KIND_MASK << ((index as u32 + 1) * KIND_BITS))
    }

    /// This value without parameter `index`'s label.
    pub const fn without_label(self, index: usize) -> Taint {
        Taint(self.0 & !Taint::param(index).0)
    }

    /// Kinds of parameter `index`'s label this value still carries.
    pub const fn param_kinds(self, index: usize) -> Taint {
        Taint((self.0 >> ((index as u32 + 1) * KIND_BITS)) & KIND_MASK)
    }
}

impl std::ops::BitOr for Taint {
    type Output = Taint;
    fn bitor(self, rhs: Taint) -> Taint {
        Taint(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Taint {
    fn bitor_assign(&mut self, rhs: Taint) {
        self.0 |= rhs.0;
    }
}

/// A parameter whose value reaches a sink inside the callee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSink {
    pub param: usize,
    pub kinds: Taint,
    pub issue: mir_issues::IssueKind,
}

/// How a function or method moves taint, seen from its call sites.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaintSummary {
    /// Taint of the return value: the source slot is taint the body reads
    /// itself, each parameter slot the kinds of that argument that flow out.
    pub returns: Taint,
    pub param_sinks: Vec<ParamSink>,
    /// Taint written to the class's own properties, keyed like
    /// [`crate::db::taint_sources::prop_key`].
    pub prop_writes: Vec<(mir_types::Name, Taint)>,
    /// Label slots past the parameters that stand for `$this`'s properties.
    pub prop_labels: Vec<(usize, mir_types::Name)>,
}

impl TaintSummary {
    pub fn is_empty(&self) -> bool {
        self.returns.is_empty() && self.param_sinks.is_empty() && self.prop_writes.is_empty()
    }

    pub fn record_prop_write(&mut self, key: mir_types::Name, taint: Taint) {
        if taint.is_empty() {
            return;
        }
        match self.prop_writes.iter_mut().find(|(k, _)| *k == key) {
            Some((_, existing)) => *existing |= taint,
            None => self.prop_writes.push((key, taint)),
        }
    }

    /// Combine with another body's summary, for a call that may run either.
    pub fn merge(&mut self, mut other: TaintSummary) {
        for (slot, prop) in std::mem::take(&mut other.prop_labels) {
            match self.prop_label(slot) {
                Some(own) if own != prop => other.drop_label(slot),
                Some(_) => {}
                None => self.prop_labels.push((slot, prop)),
            }
        }
        self.returns |= other.returns;
        for sink in other.param_sinks {
            match self
                .param_sinks
                .iter_mut()
                .find(|s| s.param == sink.param && s.issue == sink.issue)
            {
                Some(existing) => existing.kinds |= sink.kinds,
                None => self.param_sinks.push(sink),
            }
        }
        for (key, taint) in other.prop_writes {
            self.record_prop_write(key, taint);
        }
    }

    /// Source taint written to property `key` whatever the arguments.
    pub fn prop_write(&self, key: mir_types::Name) -> Taint {
        self.prop_writes
            .iter()
            .filter(|(k, _)| *k == key)
            .fold(Taint::NONE, |acc, (_, t)| acc | t.concrete())
    }

    /// Forget what flows from label slot `slot`.
    fn drop_label(&mut self, slot: usize) {
        self.returns = self.returns.without_label(slot);
        self.param_sinks.retain(|sink| sink.param != slot);
        for (_, taint) in &mut self.prop_writes {
            *taint = taint.without_label(slot);
        }
    }

    /// Taint flowing into label slot `slot` at a call: a property of the
    /// receiver, or the arguments bound to that parameter.
    fn slot_taint(
        &self,
        slot: usize,
        receiver: Option<&Expr>,
        arg_taint: impl Fn(usize) -> Taint,
        ctx: &FlowState,
    ) -> Taint {
        match self.prop_label(slot) {
            Some(prop) => receiver_prop_taint(receiver, prop, ctx),
            None => arg_taint(slot),
        }
    }

    /// The property of `$this` label slot `slot` stands for.
    pub fn prop_label(&self, slot: usize) -> Option<mir_types::Name> {
        self.prop_labels
            .iter()
            .find(|(s, _)| *s == slot)
            .map(|(_, prop)| *prop)
    }

    /// Record that `taint`, reaching a sink of `kinds`, carries parameter labels.
    pub fn record_param_sinks(
        &mut self,
        taint: Taint,
        kinds: Taint,
        issue: &mir_issues::IssueKind,
    ) {
        for param in 0..MAX_TRACKED_PARAMS {
            let reached = taint.param_kinds(param).restrict_to(kinds);
            if reached.is_empty() {
                continue;
            }
            match self
                .param_sinks
                .iter_mut()
                .find(|s| s.param == param && s.issue == *issue)
            {
                Some(existing) => existing.kinds |= reached,
                None => self.param_sinks.push(ParamSink {
                    param,
                    kinds: reached,
                    issue: issue.clone(),
                }),
            }
        }
    }

    /// Taint of a call's result, given the taint of the argument bound to each parameter.
    pub fn call_result(&self, arg_taint: impl Fn(usize) -> Taint) -> Taint {
        resolve_labels(self.returns, arg_taint)
    }
}

/// `taint` with each label slot replaced by what flows into it at a call.
pub fn resolve_labels(taint: Taint, slot_taint: impl Fn(usize) -> Taint) -> Taint {
    let mut out = taint.concrete();
    for slot in 0..MAX_TRACKED_PARAMS {
        let kinds = taint.param_kinds(slot);
        if !kinds.is_empty() {
            out |= slot_taint(slot).restrict_to(kinds);
        }
    }
    out
}

/// Take the body's summary; a generator's `return` value isn't the call result.
pub fn finish_summary(summary: &mut TaintSummary, is_generator: bool) -> TaintSummary {
    let mut summary = std::mem::take(summary);
    if is_generator {
        summary.returns = Taint::NONE;
    }
    summary
}

/// Label each parameter so the body's summary can tell which arguments flow where.
/// Only inference builds summaries; a full analysis reports source taint alone.
pub fn seed_param_labels(
    ctx: &mut FlowState,
    params: &[mir_codebase::DeclaredParam],
    mode: crate::body_analysis::AnalysisMode,
) {
    if mode == crate::body_analysis::AnalysisMode::InferenceOnly {
        ctx.summarizing_taint = true;
        label_params(ctx, params.iter().map(|p| p.name.as_ref()));
    }
}

/// Label the properties of `$this` a method body reads with the slots its
/// parameters leave free, so its summary can tell which of them flow where.
pub fn seed_prop_labels(
    ctx: &mut FlowState,
    param_count: usize,
    body: &[php_ast::owned::Stmt],
) -> Vec<(usize, mir_types::Name)> {
    if !ctx.summarizing_taint || param_count >= MAX_TRACKED_PARAMS {
        return Vec::new();
    }
    let mut reads = ThisPropReads(Vec::new());
    for stmt in body {
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut reads, stmt);
    }
    let labels: Vec<(usize, mir_types::Name)> =
        (param_count..MAX_TRACKED_PARAMS).zip(reads.0).collect();
    for (slot, prop) in &labels {
        ctx.set_prop_taint("this", prop.as_str(), Taint::param(*slot));
    }
    labels
}

/// Properties read through `$this->name`, in source order.
struct ThisPropReads(Vec<mir_types::Name>);

impl php_ast::owned::visitor::OwnedVisitor for ThisPropReads {
    fn visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
        if let ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) = &expr.kind {
            if matches!(&pa.object.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "this")
            {
                if let Some(prop) = crate::expr::helpers::extract_string_from_expr(&pa.property) {
                    let prop = mir_types::Name::new(&prop);
                    if !self.0.contains(&prop) {
                        self.0.push(prop);
                    }
                }
            }
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// Give each parameter its label, replacing whatever taint the name had.
pub fn label_params<'p>(ctx: &mut FlowState, names: impl Iterator<Item = &'p str>) {
    for (index, name) in names.take(MAX_TRACKED_PARAMS).enumerate() {
        ctx.set_var_taint(name, Taint::param(index));
    }
}

/// The analyzed class's own property a write lands in: `$this->p`, `$this->p['k']`,
/// `$this->p->q`, `self::$p`.
pub fn own_property_written(
    target: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<mir_types::Name> {
    match &target.kind {
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            match &pa.object.kind {
                ExprKind::Variable(v) if v.trim_start_matches('$') == "this" => {
                    let prop = crate::expr::helpers::extract_string_from_expr(&pa.property)?;
                    Some(crate::db::taint_sources::prop_key(&prop, false))
                }
                _ => own_property_written(&pa.object, ctx, db, file),
            }
        }
        ExprKind::ArrayAccess(aa) => own_property_written(&aa.array, ctx, db, file),
        ExprKind::StaticPropertyAccess(spa) => {
            let (fqcn, prop) =
                crate::expr::assignment::resolve_static_prop_target(spa, ctx, db, file)?;
            (ctx.self_fqcn.as_deref() == Some(fqcn.as_ref()))
                .then(|| crate::db::taint_sources::prop_key(&prop, true))
        }
        _ => None,
    }
}

/// Source taint any method of `class` or its ancestors may store in property `key`.
pub fn class_property_taint(
    db: &dyn crate::db::MirDatabase,
    from: &str,
    class: &str,
    key: mir_types::Name,
) -> Taint {
    crate::db::taint_sources::class_property_writers(db, from, class, key)
        .iter()
        .fold(Taint::NONE, |acc, (owner, method)| {
            acc | crate::db::method_taint_summary(db, from, owner.as_str(), method.as_str())
                .map_or(Taint::NONE, |s| s.prop_write(key))
        })
}

/// Expressions passed for declared parameter `index`, including a spread that may cover it.
pub fn param_arg_exprs<'e>(
    params: &[mir_codebase::DeclaredParam],
    args: &'e [php_ast::owned::Arg],
    index: usize,
) -> Vec<&'e Expr> {
    let Some(param) = params.get(index) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut position = 0;
    for arg in args {
        let Some(value) = &arg.value else {
            position += 1;
            continue;
        };
        if let Some(name) = &arg.name {
            let name = crate::parser::name_to_string_owned(name);
            let unknown_name = || !params.iter().any(|p| p.name.as_ref() == name);
            if name == param.name.as_ref() || (param.is_variadic && unknown_name()) {
                out.push(value);
            }
        } else if arg.unpack {
            if position <= index || param.is_variadic {
                out.push(value);
            }
        } else if position == index || (param.is_variadic && position >= index) {
            out.push(value);
        }
        position += 1;
    }
    out
}

/// Taint of the arguments bound to parameter `index`.
pub fn param_arg_taint(
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    index: usize,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    param_arg_exprs(params, args, index)
        .into_iter()
        .fold(Taint::NONE, |acc, e| acc | expr_taint(e, ctx, db, file))
}

/// Taint `new` leaves in each instance property of the object it creates,
/// from what the constructor stores there.
pub fn constructed_prop_taints(
    new: &php_ast::owned::NewExpr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<(mir_types::Name, Taint)> {
    let ExprKind::Identifier(name) = &new.class.kind else {
        return Vec::new();
    };
    let class = match crate::db::resolve_name(db, file, name.as_ref()).as_str() {
        "self" | "static" => ctx.self_fqcn.clone(),
        "parent" => ctx.parent_fqcn.clone(),
        resolved => Some(std::sync::Arc::from(resolved)),
    };
    let Some((owner, ctor)) = class.and_then(|class| {
        crate::db::find_method_in_chain(db, crate::db::Fqcn::from_str(db, &class), "__construct")
    }) else {
        return Vec::new();
    };
    // Like a receiver's, an object's properties only demand summaries for source data.
    if args_taint(&new.args, ctx, db, file).concrete().is_empty()
        || !crate::db::taint_sources::method_stores_param(db, file, owner.as_ref(), "__construct")
    {
        return Vec::new();
    }
    let Some(summary) = crate::db::method_taint_summary(db, file, owner.as_ref(), "__construct")
    else {
        return Vec::new();
    };
    let arg_taint = |slot: usize| match summary.prop_label(slot) {
        Some(_) => Taint::NONE,
        None => param_arg_taint(&ctor.params, &new.args, slot, ctx, db, file).concrete(),
    };
    summary
        .prop_writes
        .iter()
        .filter(|(key, _)| !key.as_str().starts_with('$'))
        .map(|(key, taint)| (*key, resolve_labels(*taint, arg_taint)))
        .filter(|(_, taint)| !taint.is_empty())
        .collect()
}

/// Union of the kinds named by taint annotations (`html`, `sql`, `input`, ...).
fn named_kinds(names: &[std::sync::Arc<str>]) -> Taint {
    names.iter().fold(Taint::NONE, |acc, name| {
        acc | match name.as_ref() {
            "input" | "text" => Taint::ALL,
            "has_quotes" => Taint::HTML | Taint::SQL,
            other => taint_sink_kind(other),
        }
    })
}

/// A return type whose values can't carry a payload into a sink.
fn return_drops_taint(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    ty.is_void()
        || ty.is_never()
        || is_injection_safe_type(ty)
        || ty.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TNamedObject { .. }
                    | Atomic::TStaticObject { .. }
                    | Atomic::TSelf { .. }
                    | Atomic::TParent { .. }
                    | Atomic::TClosure { .. }
                    | Atomic::TObject
            )
        })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CalleeBody {
    /// Abstract, interface or stub: only annotations describe it.
    None,
    Dependency,
    Summarized,
}

impl CalleeBody {
    fn of(location: Option<&mir_types::Location>, from: &str) -> CalleeBody {
        match location {
            Some(loc) if crate::stubs::is_stub_path(loc.file.as_ref()) => CalleeBody::None,
            Some(loc) if is_dependency_of(loc.file.as_ref(), from) => CalleeBody::Dependency,
            Some(_) => CalleeBody::Summarized,
            None => CalleeBody::None,
        }
    }
}

/// Taint the object `receiver` evaluates to holds in property `prop`.
fn receiver_prop_taint(receiver: Option<&Expr>, prop: mir_types::Name, ctx: &FlowState) -> Taint {
    match receiver.map(|r| &unwrap_parens(r).kind) {
        Some(ExprKind::Variable(var)) => ctx.prop_taint(var.trim_start_matches('$'), prop.as_str()),
        _ => Taint::NONE,
    }
}

/// Whether the object `receiver` evaluates to holds source data worth a callee
/// summary; `$this`'s property labels alone would demand one for every call on it.
fn receiver_demands_summary(receiver: Option<&Expr>, ctx: &FlowState) -> bool {
    let Some(ExprKind::Variable(var)) = receiver.map(|r| &unwrap_parens(r).kind) else {
        return false;
    };
    let var = mir_types::Name::from(var.trim_start_matches('$'));
    ctx.tainted_props
        .iter()
        .any(|((object, _), taint)| *object == var && !taint.concrete().is_empty())
}

/// The part of an argument's taint worth inferring a callee summary for:
/// parameter labels only matter while a summary is being built.
fn demanding_taint(taint: Taint, ctx: &FlowState) -> Taint {
    if ctx.summarizing_taint {
        taint
    } else {
        taint.concrete()
    }
}

/// Taint of a call to user code, from the callee's annotations and inferred summary.
#[allow(clippy::too_many_arguments)]
fn user_call_taint(
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    return_type: Option<&mir_types::Type>,
    annotations: Option<&mir_codebase::TaintAnnotations>,
    body: CalleeBody,
    receiver: Option<&Expr>,
    returns_param: impl FnOnce() -> bool,
    returns_prop: impl FnOnce() -> bool,
    reads_source: impl FnOnce() -> bool,
    summary: impl FnOnce() -> Option<TaintSummary>,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if return_type.is_some_and(return_drops_taint) {
        return Taint::NONE;
    }
    let arg_taint = |index| param_arg_taint(params, args, index, ctx, db, file);
    // Inferring a summary only pays off when it can return source data, tainted
    // arguments or the receiver's tainted properties.
    let worth_summary = || {
        (returns_param() && !demanding_taint(args_taint(args, ctx, db, file), ctx).is_empty())
            || (receiver_demands_summary(receiver, ctx) && returns_prop())
            || reads_source()
    };
    let mut taint = match body {
        CalleeBody::Summarized if worth_summary() => summary().map_or(Taint::NONE, |s| {
            s.call_result(|slot| s.slot_taint(slot, receiver, arg_taint, ctx))
        }),
        CalleeBody::Dependency => args_taint(args, ctx, db, file),
        _ => Taint::NONE,
    };
    let Some(annotations) = annotations else {
        return taint;
    };
    for name in &annotations.flow_params {
        if let Some(index) = params.iter().position(|p| p.name.as_ref() == name.as_ref()) {
            taint |= arg_taint(index);
        }
    }
    let escaped = named_kinds(&annotations.escape);
    let unescaped = named_kinds(&annotations.unescape);
    // An annotated escaper or decoder without a body to summarize passes its arguments through.
    if body == CalleeBody::None && annotations.flow_params.is_empty() {
        taint = args_taint(args, ctx, db, file);
    }
    taint.without(escaped).restore(unescaped)
}

/// Report a call target or class name chosen by user input (`$_GET['f']()`).
pub fn check_callable_target(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    target: &Expr,
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let issue = mir_issues::IssueKind::TaintedCallable;
    if reaches_sink(
        target,
        Taint::CALLABLE,
        &issue,
        ctx,
        ea.db,
        &ea.file,
        ea.taint_summary,
    ) {
        ea.emit_taint(issue, span, target, Taint::CALLABLE, ctx);
    }
}

/// Report arguments reaching a sink inside the callee: its `@taint-sink`
/// parameters and the sinks its inferred summary records.
#[allow(clippy::too_many_arguments)]
pub fn check_call_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    ctx: &FlowState,
    params: &[mir_codebase::DeclaredParam],
    args: &[php_ast::owned::Arg],
    receiver: Option<&Expr>,
    sink_params: &[(std::sync::Arc<str>, std::sync::Arc<str>)],
    summary: impl FnOnce(&dyn crate::db::MirDatabase, &str) -> Option<TaintSummary>,
    span: php_ast::Span,
) {
    let mut sinks: Vec<ParamSink> = sink_params
        .iter()
        .filter_map(|(name, kind)| {
            Some(ParamSink {
                param: params
                    .iter()
                    .position(|p| p.name.as_ref() == name.as_ref())?,
                kinds: taint_sink_kind(kind),
                issue: taint_sink_issue(kind),
            })
        })
        .collect();
    let mut prop_labels = Vec::new();
    if !demanding_taint(args_taint(args, ctx, ea.db, &ea.file), ctx).is_empty()
        || receiver_demands_summary(receiver, ctx)
    {
        if let Some(summary) = summary(ea.db, &ea.file) {
            sinks.extend(summary.param_sinks);
            prop_labels = summary.prop_labels;
        }
    }
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for sink in &sinks {
        if let Some((_, prop)) = prop_labels.iter().find(|(slot, _)| *slot == sink.param) {
            let (Some(receiver), taint) = (receiver, receiver_prop_taint(receiver, *prop, ctx))
            else {
                continue;
            };
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit_taint(sink.issue.clone(), span, receiver, sink.kinds, ctx);
            }
            continue;
        }
        for value in param_arg_exprs(params, args, sink.param) {
            let taint = expr_taint(value, ctx, ea.db, &ea.file);
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit_taint(sink.issue.clone(), span, value, sink.kinds, ctx);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Superglobal names (without the $ prefix, as stored in FlowState::vars)
// ---------------------------------------------------------------------------

/// Superglobals holding request data; `$_ENV` comes from the process environment.
pub const SUPERGLOBALS: &[&str] = &["_GET", "_POST", "_REQUEST", "_COOKIE", "_FILES", "_SERVER"];

pub fn is_superglobal(name: &str) -> bool {
    SUPERGLOBALS.contains(&name)
}

/// `$_SERVER` keys a client controls; the rest come from the server or environment.
fn is_request_server_key(key: &str) -> bool {
    key.starts_with("HTTP_")
        || matches!(
            key,
            "REQUEST_URI"
                | "QUERY_STRING"
                | "PHP_SELF"
                | "PATH_INFO"
                | "PATH_TRANSLATED"
                | "ORIG_PATH_INFO"
                | "REQUEST_METHOD"
                | "CONTENT_TYPE"
                | "CONTENT_LENGTH"
                | "PHP_AUTH_USER"
                | "PHP_AUTH_PW"
                | "PHP_AUTH_DIGEST"
                | "REMOTE_USER"
                | "REDIRECT_URL"
                | "REDIRECT_QUERY_STRING"
                | "UNENCODED_URL"
                | "SERVER_NAME"
        )
}

fn is_superglobal_var(expr: &Expr, name: &str) -> bool {
    matches!(&expr.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == name)
}

/// A superglobal entry PHP, the server or the environment sets rather than the
/// client: `$_SERVER['DOCUMENT_ROOT']`, `$_FILES[x]['tmp_name']`.
pub fn is_trusted_superglobal_field(aa: &php_ast::owned::ArrayAccessExpr) -> bool {
    let Some(ExprKind::String(key)) = aa.index.as_deref().map(|i| &i.kind) else {
        return false;
    };
    if is_superglobal_var(&aa.array, "_SERVER") {
        return !is_request_server_key(key);
    }
    let ExprKind::ArrayAccess(file_entry) = &aa.array.kind else {
        return false;
    };
    matches!(&**key, "tmp_name" | "size" | "error")
        && is_superglobal_var(&file_entry.array, "_FILES")
}

/// The global variable `$GLOBALS['name']` reads.
fn globals_entry(aa: &php_ast::owned::ArrayAccessExpr) -> Option<&str> {
    let Some(ExprKind::String(key)) = aa.index.as_deref().map(|i| &i.kind) else {
        return None;
    };
    is_superglobal_var(&aa.array, "GLOBALS").then_some(key.as_ref())
}

/// `$GLOBALS['_GET']` reads the superglobal itself.
pub fn is_globals_superglobal_alias(aa: &php_ast::owned::ArrayAccessExpr) -> bool {
    matches!(aa.index.as_deref().map(|i| &i.kind), Some(ExprKind::String(key)) if is_superglobal(key))
        && is_superglobal_var(&aa.array, "GLOBALS")
}

// ---------------------------------------------------------------------------
// Sink classification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SinkKind {
    Html,        // echo / print → XSS
    Sql,         // DB query functions → SQL injection
    Shell,       // system / exec / shell_exec → command injection
    File,        // filesystem path args → path traversal / local file inclusion
    Unserialize, // unserialize → PHP object injection
    Header,      // header → response splitting / open redirect
    Cookie,      // setcookie / setrawcookie → cookie injection
    Ssrf,        // connection targets → server-side request forgery
    Ldap,        // LDAP filters and DNs → LDAP injection
    Xpath,       // XPath expressions → XPath injection
    Callable,    // callbacks and dynamic calls → arbitrary code paths
}

impl SinkKind {
    pub fn issue(self) -> mir_issues::IssueKind {
        use mir_issues::IssueKind;
        match self {
            SinkKind::Html => IssueKind::TaintedHtml,
            SinkKind::Sql => IssueKind::TaintedSql,
            SinkKind::Shell => IssueKind::TaintedShell,
            SinkKind::Header => IssueKind::TaintedHeader,
            SinkKind::Cookie => IssueKind::TaintedCookie,
            SinkKind::File => IssueKind::TaintedInput {
                sink: "file".to_string(),
            },
            SinkKind::Unserialize => IssueKind::TaintedInput {
                sink: "unserialize".to_string(),
            },
            SinkKind::Ssrf => IssueKind::TaintedSsrf,
            SinkKind::Ldap => IssueKind::TaintedLdap,
            SinkKind::Xpath => IssueKind::TaintedXpath,
            SinkKind::Callable => IssueKind::TaintedCallable,
        }
    }

    pub fn taint(self) -> Taint {
        match self {
            SinkKind::Html => Taint::HTML,
            SinkKind::Sql => Taint::SQL,
            SinkKind::Shell => Taint::SHELL,
            SinkKind::File => Taint::FILE,
            SinkKind::Unserialize => Taint::UNSERIALIZE,
            SinkKind::Header => Taint::HEADER,
            SinkKind::Cookie => Taint::COOKIE,
            SinkKind::Ssrf => Taint::SSRF,
            SinkKind::Ldap => Taint::LDAP,
            SinkKind::Xpath => Taint::XPATH,
            SinkKind::Callable => Taint::CALLABLE,
        }
    }
}

/// One dangerous parameter of a builtin sink. `index: None` means every argument.
#[derive(Debug, Clone, Copy)]
pub struct SinkParam {
    pub index: Option<usize>,
    pub name: &'static str,
    pub kind: SinkKind,
}

const fn any(kind: SinkKind) -> SinkParam {
    SinkParam {
        index: None,
        name: "",
        kind,
    }
}

const fn at(index: usize, name: &'static str, kind: SinkKind) -> SinkParam {
    SinkParam {
        index: Some(index),
        name,
        kind,
    }
}

macro_rules! sinks {
    ($($param:expr),* $(,)?) => {{
        const PARAMS: &[SinkParam] = &[$($param),*];
        PARAMS
    }};
}

/// The parameters of builtin `fn_name` that are taint sinks.
pub fn builtin_sinks(fn_name: &str) -> &'static [SinkParam] {
    use SinkKind::*;
    match crate::util::php_ident_lowercase(fn_name.trim_start_matches('\\')).as_str() {
        "echo" | "print" | "printf" | "vprintf" | "fprintf" | "vfprintf" => sinks![any(Html)],

        "header" => sinks![at(0, "header", Header)],
        "setcookie" | "setrawcookie" => sinks![any(Cookie)],
        "mail" => sinks![
            at(3, "additional_headers", Header),
            at(4, "additional_params", Shell),
        ],

        "mysql_query"
        | "mysqli_query"
        | "mysqli_real_query"
        | "mysqli_multi_query"
        | "mysql_unbuffered_query"
        | "mysql_db_query"
        | "mysqli_prepare"
        | "mysqli_stmt_prepare"
        | "pg_query"
        | "pg_send_query"
        | "pg_prepare"
        | "pg_exec"
        | "sqlite_query"
        | "sqlite_exec"
        | "sqlite_array_query"
        | "sqlite_single_query"
        | "sqlite_unbuffered_query"
        | "mssql_query"
        | "odbc_exec"
        | "odbc_prepare"
        | "oci_parse"
        | "db2_exec"
        | "db2_prepare" => sinks![any(Sql)],

        "system" | "exec" | "shell_exec" | "passthru" | "popen" | "proc_open" | "expect_popen" => {
            sinks![at(0, "command", Shell)]
        }
        "pcntl_exec" => sinks![at(0, "path", Shell), at(1, "args", Shell)],
        "ssh2_exec" => sinks![at(1, "command", Shell)],

        "fopen"
        | "file_get_contents"
        | "file_put_contents"
        | "readfile"
        | "file"
        | "unlink"
        | "touch"
        | "chmod"
        | "chown"
        | "parse_ini_file"
        | "simplexml_load_file"
        | "highlight_file"
        | "show_source"
        | "php_strip_whitespace"
        | "getimagesize"
        | "gzopen"
        | "gzfile"
        | "readgzfile" => sinks![at(0, "filename", File)],
        "mkdir" | "rmdir" | "scandir" | "opendir" | "chdir" => sinks![at(0, "directory", File)],
        "copy" | "rename" => sinks![at(0, "from", File), at(1, "to", File)],
        "symlink" | "link" => sinks![at(0, "target", File), at(1, "link", File)],
        "readlink" => sinks![at(0, "path", File)],
        "glob" => sinks![at(0, "pattern", File)],
        "dl" => sinks![at(0, "extension_filename", File)],
        "move_uploaded_file" => sinks![at(1, "to", File)],

        "fsockopen" | "pfsockopen" => sinks![at(0, "hostname", Ssrf)],
        "get_headers" | "curl_init" => sinks![at(0, "url", Ssrf)],
        "stream_socket_client" => sinks![at(0, "address", Ssrf)],

        "ldap_search" | "ldap_list" | "ldap_read" => {
            sinks![at(1, "base", Ldap), at(2, "filter", Ldap)]
        }
        "ldap_delete" | "ldap_compare" => sinks![at(1, "dn", Ldap)],

        "unserialize" => sinks![at(0, "data", Unserialize)],

        "call_user_func"
        | "call_user_func_array"
        | "forward_static_call"
        | "forward_static_call_array"
        | "register_shutdown_function"
        | "register_tick_function"
        | "spl_autoload_register"
        | "set_error_handler"
        | "set_exception_handler"
        | "array_map"
        | "ob_start" => sinks![at(0, "callback", Callable)],
        "usort"
        | "uasort"
        | "uksort"
        | "array_filter"
        | "array_walk"
        | "array_walk_recursive"
        | "array_reduce"
        | "iterator_apply"
        | "preg_replace_callback" => {
            sinks![at(1, "callback", Callable)]
        }

        _ => sinks![],
    }
}

/// The first sink kind of builtin `fn_name`, if it is a sink.
pub fn classify_sink(fn_name: &str) -> Option<SinkKind> {
    builtin_sinks(fn_name).first().map(|p| p.kind)
}

/// Values reaching sinks that depend on another argument: `curl_setopt($ch,
/// CURLOPT_URL, $url)`, `curl_setopt_array($ch, [CURLOPT_URL => $url])`.
pub fn option_sink_values<'e>(
    fn_name_lower: &str,
    args: &'e [php_ast::owned::Arg],
) -> Vec<(&'e Expr, SinkKind)> {
    let is_target_option = |e: &Expr| {
        matches!(&e.kind, ExprKind::Identifier(name)
            if matches!(name.trim_start_matches('\\'), "CURLOPT_URL" | "CURLOPT_PROXY"))
    };
    match fn_name_lower.trim_start_matches('\\') {
        "curl_setopt" => match (args.get(1).and_then(|a| a.value.as_ref()), args.get(2)) {
            (Some(option), Some(value)) if is_target_option(option) => {
                value.value.iter().map(|v| (v, SinkKind::Ssrf)).collect()
            }
            _ => Vec::new(),
        },
        "curl_setopt_array" => match args.get(1).and_then(|a| a.value.as_ref()).map(|v| &v.kind) {
            Some(ExprKind::Array(elements)) => elements
                .iter()
                .filter(|el| el.key.as_ref().is_some_and(is_target_option))
                .map(|el| (&el.value, SinkKind::Ssrf))
                .collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    }
}

/// `unserialize($data, ['allowed_classes' => false])` can't instantiate objects.
pub fn unserialize_disallows_classes(args: &[php_ast::owned::Arg]) -> bool {
    let Some(ExprKind::Array(options)) =
        args.get(1).and_then(|a| a.value.as_ref()).map(|v| &v.kind)
    else {
        return false;
    };
    options.iter().any(|el| {
        matches!(el.key.as_ref().map(|k| &k.kind), Some(ExprKind::String(k)) if &**k == "allowed_classes")
            && matches!(&el.value.kind, ExprKind::Bool(false))
    })
}

const SYMFONY_REQUEST: &str = "Symfony\\Component\\HttpFoundation\\Request";

/// `Request` properties holding client data (`$request->query->get('q')`).
/// `attributes` is left out: the router fills it (`_controller`, `_route`).
const REQUEST_BAG_PROPERTIES: &[&str] =
    &["query", "request", "cookies", "headers", "files", "server"];

/// Request accessors returning client data: Symfony's `Request`, Laravel's
/// subclass, and PSR-7 requests.
pub fn is_framework_request_source(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
) -> bool {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    if is_a(SYMFONY_REQUEST) {
        return matches!(
            method_lower,
            "get"
                | "getcontent"
                | "getpayload"
                | "toarray"
                | "geturi"
                | "getrequesturi"
                | "getpathinfo"
                | "getquerystring"
                | "gethost"
                | "gethttphost"
                | "getschemeandhttphost"
                | "getbaseurl"
                | "getbasepath"
                | "getuserinfo"
                | "getuser"
                | "getpassword"
                | "getclientip"
                | "getclientips"
                | "getpreferredlanguage"
                | "getlanguages"
                | "getcharsets"
                | "getencodings"
                | "getacceptablecontenttypes"
                | "getetags"
                | "getrequestformat"
        ) || is_laravel_request_accessor(method_lower);
    }
    if is_a("Psr\\Http\\Message\\ServerRequestInterface")
        && matches!(
            method_lower,
            "getqueryparams"
                | "getparsedbody"
                | "getcookieparams"
                | "getuploadedfiles"
                | "getserverparams"
        )
    {
        return true;
    }
    is_a("Psr\\Http\\Message\\RequestInterface")
        && matches!(
            method_lower,
            "getheader"
                | "getheaderline"
                | "getheaders"
                | "getbody"
                | "geturi"
                | "getrequesttarget"
        )
}

fn is_laravel_request_accessor(method_lower: &str) -> bool {
    matches!(
        method_lower,
        "input"
            | "query"
            | "post"
            | "cookie"
            | "header"
            | "server"
            | "all"
            | "only"
            | "except"
            | "json"
            | "string"
            | "str"
            | "keys"
            | "collect"
            | "route"
            | "fullurl"
            | "fullurlwithquery"
            | "fullurlwithoutquery"
            | "url"
            | "path"
            | "decodedpath"
            | "segment"
            | "segments"
            | "ip"
            | "ips"
            | "useragent"
            | "bearertoken"
            | "validated"
            | "safe"
            | "old"
            | "file"
            | "allfiles"
    )
}

/// `Request::input('q')` through Laravel's facades.
pub fn is_laravel_request_facade_source(class: &str, method_lower: &str) -> bool {
    matches!(
        class.trim_start_matches('\\'),
        "Request"
            | "Input"
            | "Illuminate\\Support\\Facades\\Request"
            | "Illuminate\\Support\\Facades\\Input"
    ) && (method_lower == "get" || is_laravel_request_accessor(method_lower))
}

/// `$request->query` and the other client-data bags of a Symfony request.
pub fn is_request_bag_property(db: &dyn crate::db::MirDatabase, fqcn: &str, prop: &str) -> bool {
    REQUEST_BAG_PROPERTIES.contains(&prop)
        && (fqcn.eq_ignore_ascii_case(SYMFONY_REQUEST)
            || crate::db::extends_or_implements(db, fqcn, SYMFONY_REQUEST))
}

/// Laravel's `request('q')` helper reads input; `request()` returns the request object.
pub fn is_request_helper_source(fn_name_lower: &str, args: &[php_ast::owned::Arg]) -> bool {
    fn_name_lower.trim_start_matches('\\') == "request" && !args.is_empty()
}

/// Return the sink kind for a *method* call on a known OOP database wrapper —
/// `$pdo->query($sql)`, `$mysqli->prepare($sql)` — the dominant modern PHP
/// idiom over the procedural `mysqli_query()` style `classify_sink` covers.
/// Matches the class itself or any subclass (a custom `Database extends PDO`
/// is just as much a sink).
pub fn classify_method_sink(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_name: &str,
) -> Option<SinkKind> {
    let method = crate::util::php_ident_lowercase(method_name);
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };

    if is_a("PDO") && matches!(method.as_str(), "query" | "exec" | "prepare") {
        return Some(SinkKind::Sql);
    }
    if is_a("mysqli")
        && matches!(
            method.as_str(),
            "query" | "prepare" | "real_query" | "multi_query"
        )
    {
        return Some(SinkKind::Sql);
    }
    if is_a("SQLite3") && matches!(method.as_str(), "query" | "exec" | "prepare") {
        return Some(SinkKind::Sql);
    }
    if is_a("DOMXPath") && matches!(method.as_str(), "query" | "evaluate") {
        return Some(SinkKind::Xpath);
    }
    if is_a("SimpleXMLElement") && method == "xpath" {
        return Some(SinkKind::Xpath);
    }
    None
}

/// Constructors whose first argument is a sink: `new SplFileObject($path)`.
pub fn classify_constructor_sink(db: &dyn crate::db::MirDatabase, fqcn: &str) -> Option<SinkKind> {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    if ["SplFileObject", "DirectoryIterator", "GlobIterator"]
        .iter()
        .any(|base| is_a(base))
    {
        return Some(SinkKind::File);
    }
    is_a("SoapClient").then_some(SinkKind::Ssrf)
}

/// Map a `@taint-sink <kind> $param` docblock tag's free-text kind to the
/// issue it should raise. Named kinds reuse the issue the matching built-in
/// sink raises (`classify_sink` / `classify_method_sink`); anything else,
/// including typos, falls back to the generic `TaintedInput`, as
/// `classify_sink`'s File/Unserialize arms do.
pub fn taint_sink_issue(kind: &str) -> mir_issues::IssueKind {
    match kind {
        "llm_prompt" => mir_issues::IssueKind::TaintedLlmPrompt,
        "html" => mir_issues::IssueKind::TaintedHtml,
        "ssrf" => mir_issues::IssueKind::TaintedSsrf,
        "ldap" => mir_issues::IssueKind::TaintedLdap,
        "xpath" => mir_issues::IssueKind::TaintedXpath,
        "callable" => mir_issues::IssueKind::TaintedCallable,
        "header" => mir_issues::IssueKind::TaintedHeader,
        "cookie" => mir_issues::IssueKind::TaintedCookie,
        "sql" => mir_issues::IssueKind::TaintedSql,
        "shell" => mir_issues::IssueKind::TaintedShell,
        other => mir_issues::IssueKind::TaintedInput {
            sink: other.to_string(),
        },
    }
}

/// The taint kind a `@taint-sink <kind>` parameter checks for.
pub fn taint_sink_kind(kind: &str) -> Taint {
    match kind {
        "llm_prompt" => Taint::LLM_PROMPT,
        "html" => Taint::HTML,
        "header" => Taint::HEADER,
        "cookie" => Taint::COOKIE,
        "sql" => Taint::SQL,
        "shell" => Taint::SHELL,
        "file" | "include" => Taint::FILE,
        "unserialize" => Taint::UNSERIALIZE,
        "eval" => Taint::EVAL,
        "callable" => Taint::CALLABLE,
        "ssrf" => Taint::SSRF,
        "ldap" => Taint::LDAP,
        "xpath" => Taint::XPATH,
        _ => Taint::OTHER,
    }
}

/// Whether `expr` carries source taint the given sink kind cares about.
/// Parameter labels reaching the sink are recorded in `summary` instead.
#[allow(clippy::too_many_arguments)]
pub fn reaches_sink(
    expr: &Expr,
    sink: Taint,
    issue: &mir_issues::IssueKind,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
    summary: &mut TaintSummary,
) -> bool {
    let taint = expr_taint(expr, ctx, db, file);
    summary.record_param_sinks(taint, sink, issue);
    taint.concrete().intersects(sink)
}

// ---------------------------------------------------------------------------
// Expression taint checker
// ---------------------------------------------------------------------------

/// A value of this type can't carry an injection payload: numbers, booleans,
/// numeric strings, or string literals the program itself spells out (e.g. after
/// `is_numeric($x)` or `in_array($x, ['a', 'b'], true)` narrowing).
pub fn is_injection_safe_type(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    !ty.types.is_empty()
        && ty.types.iter().all(|a| {
            matches!(
                a,
                Atomic::TInt
                    | Atomic::TLiteralInt(_)
                    | Atomic::TIntRange { .. }
                    | Atomic::TPositiveInt
                    | Atomic::TNegativeInt
                    | Atomic::TNonNegativeInt
                    | Atomic::TFloat
                    | Atomic::TIntegralFloat
                    | Atomic::TLiteralFloat(..)
                    | Atomic::TBool
                    | Atomic::TTrue
                    | Atomic::TFalse
                    | Atomic::TNull
                    | Atomic::TNumeric
                    | Atomic::TNumericString
                    | Atomic::TLiteralString(_)
            )
        })
}

/// Whether every array shape `ty` can be holds an injection-safe value at `key`.
fn shape_entry_is_safe(ty: &mir_types::Type, key: mir_types::Name) -> bool {
    use mir_types::atomic::ArrayKey;
    !ty.types.is_empty()
        && ty.types.iter().all(|atom| match atom {
            mir_types::Atomic::TKeyedArray { properties, .. } => properties
                .iter()
                .find(|(k, _)| match k {
                    ArrayKey::String(s) => s.as_ref() == key.as_str(),
                    ArrayKey::Int(i) => i.to_string() == key.as_str(),
                })
                .is_some_and(|(_, prop)| is_injection_safe_type(&prop.ty)),
            _ => false,
        })
}

/// Strip any number of redundant parens (`((expr))` -> `expr`) — a call
/// target keeps its parens in the AST (`(fn() => ...)()`), unlike most
/// other expression positions.
fn unwrap_parens(expr: &Expr) -> &Expr {
    let mut e = expr;
    while let ExprKind::Parenthesized(inner) = &e.kind {
        e = inner;
    }
    e
}

/// Union of the taint every argument of a call carries.
pub fn args_taint(
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    args.iter()
        .filter_map(|a| a.value.as_ref())
        .fold(Taint::NONE, |acc, v| acc | expr_taint(v, ctx, db, file))
}

/// The kinds of attacker-controlled data the expression could carry, given the
/// current `FlowState` taint state.
///
/// This is a conservative over-approximation:
/// - Any reference to a superglobal (or an array offset thereof) is tainted.
/// - Any variable that was previously marked tainted in `ctx.tainted_vars` is tainted.
/// - Binary string-concat or arithmetic on tainted operands propagates taint.
/// - Interpolated strings are tainted if any embedded variable is tainted.
/// - A call to a `@taint-source`-annotated function/method is tainted.
/// - A sanitizer (`htmlspecialchars`, `escapeshellarg`, …) removes only its own kinds.
pub fn expr_taint(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    match &expr.kind {
        ExprKind::Variable(name) => {
            let n = name.trim_start_matches('$');
            if is_superglobal(n) {
                return Taint::ALL;
            }
            let sym = mir_types::Name::from(n);
            if ctx
                .vars
                .get(&sym)
                .is_some_and(|ty| is_injection_safe_type(ty))
            {
                return Taint::NONE;
            }
            // A variable with no tracked definition may be one `extract()`
            // defined from a tainted array; its name isn't known statically.
            if ctx.has_dynamic_tainted_var_def && !ctx.var_is_defined_sym(sym) {
                return Taint::ALL;
            }
            ctx.var_taint(n)
        }

        // `$$name` where `$name` holds a known literal string resolves to
        // that variable's own taint state.
        ExprKind::VariableVariable(inner) => {
            let Some(var_name) = crate::expr::helpers::extract_simple_var(inner) else {
                return Taint::NONE;
            };
            ctx.get_var(&var_name)
                .types
                .iter()
                .fold(Taint::NONE, |acc, atomic| match atomic {
                    mir_types::Atomic::TLiteralString(accessed) => {
                        acc | ctx.var_taint(accessed.as_ref())
                    }
                    _ => acc,
                })
        }

        ExprKind::ArrayAccess(aa) => {
            if is_trusted_superglobal_field(aa) {
                Taint::NONE
            } else if is_globals_superglobal_alias(aa) {
                Taint::ALL
            } else if let Some(var) = globals_entry(aa) {
                crate::db::global_var_taint(db, file, var)
            } else if let (ExprKind::Variable(name), Some(key)) = (
                &aa.array.kind,
                crate::expr::assignment::literal_key_name(aa.index.as_deref()),
            ) {
                let n = name.trim_start_matches('$');
                if is_superglobal(n) {
                    Taint::ALL
                } else if ctx
                    .vars
                    .get(&mir_types::Name::from(n))
                    .is_some_and(|ty| shape_entry_is_safe(ty, key))
                {
                    Taint::NONE
                } else {
                    ctx.key_taint(n, key)
                }
            } else {
                expr_taint(&aa.array, ctx, db, file)
            }
        }

        // The error-suppression operator only silences a notice, it doesn't
        // sanitize the value.
        ExprKind::Parenthesized(inner) | ExprKind::ErrorSuppress(inner) => {
            expr_taint(inner, ctx, db, file)
        }

        // Only a simple-variable receiver is tracked, matching prop_refined's
        // narrowing scope.
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            let Some(prop_name) = crate::expr::helpers::extract_string_from_expr(&pa.property)
            else {
                return expr_taint(&pa.object, ctx, db, file);
            };
            if let Some(recv_ty) =
                crate::expr::assignment::resolve_chained_receiver_type(&pa.object, ctx, db, file)
            {
                let is_bag = recv_ty.types.iter().any(|atom| {
                    matches!(atom, mir_types::Atomic::TNamedObject { fqcn, .. }
                        if is_request_bag_property(db, fqcn.as_ref(), &prop_name))
                });
                if is_bag {
                    return Taint::ALL;
                }
            }
            // A property of a tainted object (`json_decode($_GET['j'])->name`) is tainted too.
            let own = match &pa.object.kind {
                ExprKind::Variable(obj_var) => {
                    ctx.prop_taint(obj_var.trim_start_matches('$'), &prop_name)
                }
                _ => Taint::NONE,
            };
            let classes = receiver_classes(&pa.object, ctx, db, file);
            let declared_safe = !classes.is_empty()
                && classes.iter().all(|class| {
                    crate::db::find_property_in_chain(
                        db,
                        crate::db::Fqcn::from_str(db, class.as_ref()),
                        &prop_name,
                    )
                    .is_some_and(|(_, def)| def.ty.as_deref().is_some_and(is_injection_safe_type))
                });
            if declared_safe {
                return Taint::NONE;
            }
            let key = crate::db::taint_sources::prop_key(&prop_name, false);
            let stored = classes.iter().fold(Taint::NONE, |acc, class| {
                acc | class_property_taint(db, file, class, key)
            });
            own | stored | expr_taint(&pa.object, ctx, db, file)
        }

        ExprKind::Assign(a) => expr_taint(&a.value, ctx, db, file),

        ExprKind::Binary(op) if op.op == php_ast::ast::BinaryOp::Pipe => {
            expr_taint(&crate::expr::pipe_as_call(op, expr.span), ctx, db, file)
        }

        ExprKind::Binary(op) => {
            expr_taint(&op.left, ctx, db, file) | expr_taint(&op.right, ctx, db, file)
        }

        ExprKind::UnaryPrefix(u) => expr_taint(&u.operand, ctx, db, file),

        ExprKind::InterpolatedString(parts)
        | ExprKind::Heredoc { parts, .. }
        | ExprKind::ShellExec(parts) => parts.iter().fold(Taint::NONE, |acc, p| match p {
            StringPart::Expr(e) => acc | expr_taint(e, ctx, db, file),
            StringPart::Literal(_) => acc,
        }),

        // Short ternary (`$x ?: $y`): the true branch's value is the condition itself.
        ExprKind::Ternary(t) => {
            let then_taint = match &t.then_expr {
                Some(then_e) => expr_taint(then_e, ctx, db, file),
                None => expr_taint(&t.condition, ctx, db, file),
            };
            then_taint | expr_taint(&t.else_expr, ctx, db, file)
        }

        ExprKind::NullCoalesce(nc) => {
            expr_taint(&nc.left, ctx, db, file) | expr_taint(&nc.right, ctx, db, file)
        }

        // Numeric/boolean casts sanitize; string/array/object casts keep the payload.
        ExprKind::Cast(kind, inner) => match kind {
            php_ast::ast::CastKind::Int
            | php_ast::ast::CastKind::Float
            | php_ast::ast::CastKind::Bool => Taint::NONE,
            _ => expr_taint(inner, ctx, db, file),
        },

        ExprKind::Match(m) => m.arms.iter().fold(Taint::NONE, |acc, arm| {
            acc | expr_taint(&arm.body, ctx, db, file)
        }),

        // An element's key can carry taint just as much as its value.
        ExprKind::Array(elements) => elements.iter().fold(Taint::NONE, |acc, el| {
            let key = el
                .key
                .as_ref()
                .map_or(Taint::NONE, |k| expr_taint(k, ctx, db, file));
            acc | key | expr_taint(&el.value, ctx, db, file)
        }),

        ExprKind::FunctionCall(fc) => {
            if let Some(closure) = callee_closure(&fc.name, ctx, db, file) {
                return closure_call_taint(&closure, &fc.args, ctx, db, file);
            }
            if let ExprKind::Identifier(name) = &fc.name.kind {
                let lower_name =
                    crate::util::php_ident_lowercase(name.as_ref().trim_start_matches('\\'));
                if matches!(
                    lower_name.as_str(),
                    "getallheaders" | "apache_request_headers"
                ) {
                    return Taint::ALL;
                }
                if reads_raw_request_body(&lower_name, &fc.args)
                    || is_request_helper_source(&lower_name, &fc.args)
                {
                    return Taint::ALL;
                }
                if let Some(taint) = filter_call_taint(&lower_name, &fc.args, ctx, db, file) {
                    return taint;
                }
                if let Some(removed) = sanitized_kinds(&lower_name) {
                    return args_taint(&fc.args, ctx, db, file).without(removed);
                }
                if let Some(restored) = unescaped_kinds(&lower_name) {
                    return args_taint(&fc.args, ctx, db, file).restore(restored);
                }
                if let Some(text) = format_text_args(&lower_name, &fc.args) {
                    return text
                        .into_iter()
                        .fold(Taint::NONE, |acc, e| acc | expr_taint(e, ctx, db, file));
                }
                // `JSON_HEX_TAG` encodes `<` and `>`, so the JSON can't open a tag.
                if lower_name == "json_encode" {
                    let taint = args_taint(&fc.args[..fc.args.len().min(1)], ctx, db, file);
                    let hex_tag = fc
                        .args
                        .get(1)
                        .and_then(|a| a.value.as_ref())
                        .is_some_and(|flags| has_flag(flags, "JSON_HEX_TAG"));
                    return if hex_tag {
                        taint.without(Taint::HTML)
                    } else {
                        taint
                    };
                }
                if let Some(taint) = callback_builtin_taint(&lower_name, &fc.args, ctx, db, file) {
                    return taint;
                }
                // `compact('id')` copies `$id`'s current value into the
                // returned array; only literal names are resolved, matching
                // `compact_return_type`.
                if lower_name == "compact" {
                    return fc.args.iter().filter_map(|a| a.value.as_ref()).fold(
                        Taint::NONE,
                        |acc, v| match &v.kind {
                            ExprKind::String(var_name) => acc | ctx.var_taint(var_name.as_ref()),
                            _ => acc,
                        },
                    );
                }
                let mut resolved = crate::db::resolve_name(db, file, name.as_ref());
                let mut func = crate::db::find_function_from(
                    db,
                    file,
                    crate::db::Fqcn::from_str(db, &resolved),
                );
                // An unqualified call falls back to the global function.
                if func.is_none() && !name.contains('\\') && resolved.contains('\\') {
                    resolved = name.to_string();
                    func = crate::db::find_function_from(
                        db,
                        file,
                        crate::db::Fqcn::from_str(db, &resolved),
                    );
                }
                if func.as_ref().is_some_and(|f| f.is_taint_source) {
                    return Taint::ALL;
                }
                if let Some(f) = func.as_ref().filter(|f| is_user_code(f.location.as_ref())) {
                    return user_call_taint(
                        &f.params,
                        &fc.args,
                        f.return_type.as_deref(),
                        f.taint.as_deref(),
                        CalleeBody::of(f.location.as_ref(), file),
                        None,
                        || {
                            crate::db::taint_sources::function_returns_param(
                                db,
                                file,
                                f.fqn.as_ref(),
                            )
                        },
                        || false,
                        || {
                            crate::db::taint_sources::function_reads_source(
                                db,
                                file,
                                f.fqn.as_ref(),
                            )
                        },
                        || crate::db::function_taint_summary(db, file, f.fqn.as_ref()),
                        ctx,
                        db,
                        file,
                    );
                }
                // A builtin's result is derived from its arguments unless it can
                // only return values that carry no payload (`strlen`, `is_numeric`).
                let global = resolved.trim_start_matches('\\');
                if !global.contains('\\')
                    && crate::stubs::is_builtin_function(&crate::util::php_ident_lowercase(global))
                {
                    let safe_return = func
                        .as_ref()
                        .and_then(|f| f.return_type.as_deref())
                        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty));
                    // A path sink returns file data, not the path; a tainted path
                    // is reported at the call itself.
                    let reads_path = classify_sink(&lower_name) == Some(SinkKind::File);
                    if !safe_return && !reads_path {
                        return args_taint(&fc.args, ctx, db, file);
                    }
                }
            }
            Taint::NONE
        }

        ExprKind::StaticPropertyAccess(spa) => crate::expr::assignment::resolve_static_prop_target(
            spa, ctx, db, file,
        )
        .map_or(Taint::NONE, |(fqcn, prop_name)| {
            let key = crate::db::taint_sources::prop_key(&prop_name, true);
            ctx.static_prop_taint(&fqcn, &prop_name) | class_property_taint(db, file, &fqcn, key)
        }),

        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            // A tainted object's methods return its data (`$request->query->get('q')`).
            let receiver = expr_taint(&mc.object, ctx, db, file);
            let ExprKind::Identifier(method_name) = &mc.method.kind else {
                return receiver;
            };
            let Some(recv_ty) =
                crate::expr::assignment::resolve_chained_receiver_type(&mc.object, ctx, db, file)
            else {
                return receiver;
            };
            let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
            let mut taint = Taint::NONE;
            let mut receiver_flows = true;
            for atom in &recv_ty.types {
                let mir_types::Atomic::TNamedObject { fqcn, .. } = atom else {
                    continue;
                };
                // Laravel's `route()` without a key returns the `Route` object.
                let route_object = method_lower == "route" && mc.args.is_empty();
                if !route_object && is_framework_request_source(db, fqcn.as_ref(), &method_lower) {
                    return Taint::ALL;
                }
                if let Some(removed) = sanitized_method_kinds(db, fqcn.as_ref(), &method_lower) {
                    return args_taint(&mc.args, ctx, db, file).without(removed);
                }
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                if let Some((_, m)) =
                    crate::db::find_method_respecting_precedence(db, here, &method_lower)
                {
                    receiver_flows &= !m
                        .return_type
                        .as_deref()
                        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty));
                }
                if let Some(t) = method_call_taint(
                    here,
                    &method_lower,
                    &mc.args,
                    Some(&mc.object),
                    ctx,
                    db,
                    file,
                ) {
                    taint |= t;
                }
            }
            if receiver_flows {
                taint |= receiver;
            }
            taint
        }

        ExprKind::StaticMethodCall(smc) => {
            let fqcn_opt: Option<std::sync::Arc<str>> = if let ExprKind::Identifier(id) =
                &smc.class.kind
            {
                let resolved = crate::db::resolve_name(db, file, id.as_ref());
                match resolved.as_str() {
                    "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone()),
                    "parent" => ctx.parent_fqcn.clone(),
                    s => Some(std::sync::Arc::from(s)),
                }
            } else {
                // `$class::method()` / `$this::method()`.
                crate::expr::assignment::resolve_chained_receiver_type(&smc.class, ctx, db, file)
                    .and_then(|ty| {
                        ty.types.iter().find_map(|atom| match atom {
                            mir_types::Atomic::TClassString(Some(fqcn))
                            | mir_types::Atomic::TNamedObject { fqcn, .. }
                            | mir_types::Atomic::TSelf { fqcn }
                            | mir_types::Atomic::TStaticObject { fqcn }
                            | mir_types::Atomic::TParent { fqcn } => {
                                Some(std::sync::Arc::from(fqcn.as_ref()))
                            }
                            _ => None,
                        })
                    })
            };
            if let (Some(fqcn), ExprKind::Identifier(method_name)) = (fqcn_opt, &smc.method.kind) {
                let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
                if is_laravel_request_facade_source(fqcn.as_ref(), &method_lower) {
                    return Taint::ALL;
                }
                let here = crate::db::Fqcn::from_str(db, fqcn.as_ref());
                if let Some(taint) =
                    method_call_taint(here, &method_lower, &smc.args, None, ctx, db, file)
                {
                    return taint;
                }
            }
            Taint::NONE
        }

        _ => Taint::NONE,
    }
}

/// Classes an object expression may be an instance of.
fn receiver_classes(
    object: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<std::sync::Arc<str>> {
    if matches!(&object.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "this") {
        return ctx.self_fqcn.iter().cloned().collect();
    }
    crate::expr::assignment::resolve_chained_receiver_type(object, ctx, db, file)
        .map(|ty| {
            ty.types
                .iter()
                .filter_map(|atom| match atom {
                    mir_types::Atomic::TNamedObject { fqcn, .. } => {
                        Some(std::sync::Arc::from(fqcn.as_ref()))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

const MAX_TRAIL_STEPS: usize = 6;

fn trail_step(expr: &Expr, source: &str) -> crate::flow_state::TrailStep {
    let text = source
        .get(expr.span.start as usize..expr.span.end as usize)
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let label = match text.char_indices().nth(60) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text,
    };
    crate::flow_state::TrailStep {
        label: std::sync::Arc::from(label),
        span: expr.span,
    }
}

/// How `kinds` of source data reach `expr`, source first; empty when it carries none.
pub fn taint_trail(
    expr: &Expr,
    kinds: Taint,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
    source: &str,
) -> Vec<crate::flow_state::TrailStep> {
    if !expr_taint(expr, ctx, db, file).concrete().intersects(kinds) {
        return Vec::new();
    }
    let child = |e: &Expr| taint_trail(e, kinds, ctx, db, file, source);
    let first =
        |exprs: &mut dyn Iterator<Item = &Expr>| exprs.map(child).find(|trail| !trail.is_empty());
    let nested = match &expr.kind {
        ExprKind::Variable(name) => {
            let sym = mir_types::Name::from(name.trim_start_matches('$'));
            ctx.taint_trails.get(&sym).map(|trail| trail.to_vec())
        }
        ExprKind::Parenthesized(inner)
        | ExprKind::ErrorSuppress(inner)
        | ExprKind::Cast(_, inner)
        | ExprKind::Print(inner) => Some(child(inner)),
        ExprKind::Assign(a) => Some(child(&a.value)),
        ExprKind::ArrayAccess(aa) => match &aa.array.kind {
            ExprKind::Variable(name) if is_superglobal(name.trim_start_matches('$')) => None,
            _ => Some(child(&aa.array)),
        },
        ExprKind::Binary(op) => first(&mut [&*op.left, &*op.right].into_iter()),
        ExprKind::NullCoalesce(nc) => first(&mut [&*nc.left, &*nc.right].into_iter()),
        ExprKind::Ternary(t) => first(
            &mut [
                t.then_expr.as_deref().unwrap_or(&t.condition),
                &*t.else_expr,
            ]
            .into_iter(),
        ),
        ExprKind::InterpolatedString(parts)
        | ExprKind::Heredoc { parts, .. }
        | ExprKind::ShellExec(parts) => first(&mut parts.iter().filter_map(|p| match p {
            StringPart::Expr(e) => Some(e),
            StringPart::Literal(_) => None,
        })),
        ExprKind::Array(elements) => first(
            &mut elements
                .iter()
                .flat_map(|el| el.key.iter().chain(std::iter::once(&el.value))),
        ),
        ExprKind::FunctionCall(fc) => first(&mut fc.args.iter().filter_map(|a| a.value.as_ref()))
            .map(|mut trail| {
                trail.push(trail_step(expr, source));
                trail
            }),
        _ => None,
    };
    match nested.filter(|trail| !trail.is_empty()) {
        Some(trail) => trail,
        None => vec![trail_step(expr, source)],
    }
}

/// `trail` extended by the variable `target` now holding its data.
pub fn extend_trail(
    mut trail: Vec<crate::flow_state::TrailStep>,
    target: &Expr,
    source: &str,
) -> std::sync::Arc<[crate::flow_state::TrailStep]> {
    trail.push(trail_step(target, source));
    if trail.len() > MAX_TRAIL_STEPS {
        trail.drain(1..trail.len() - (MAX_TRAIL_STEPS - 1));
    }
    std::sync::Arc::from(trail)
}

impl crate::expr::ExpressionAnalyzer<'_> {
    /// How `kinds` of source data reach `value`, as issue trace steps.
    pub fn taint_trace(
        &self,
        value: &Expr,
        kinds: Taint,
        ctx: &FlowState,
    ) -> Option<Box<[mir_issues::TaintStep]>> {
        let trail = taint_trail(value, kinds, ctx, self.db, &self.file, self.source);
        (!trail.is_empty()).then(|| {
            trail
                .iter()
                .map(|step| mir_issues::TaintStep {
                    label: step.label.to_string(),
                    location: self.span_location(step.span),
                })
                .collect()
        })
    }

    /// Emit a taint issue, with how `value`'s data got there.
    pub fn emit_taint(
        &mut self,
        kind: mir_issues::IssueKind,
        span: php_ast::Span,
        value: &Expr,
        kinds: Taint,
        ctx: &FlowState,
    ) {
        let mut issue = self.build_issue(kind, mir_issues::Severity::Error, span);
        issue.taint_trace = self.taint_trace(value, kinds, ctx);
        self.issues.add(issue);
    }
}

/// Whether a call in `expr` takes a `function () {}` literal argument.
pub fn has_closure_argument(expr: &Expr) -> bool {
    let args = match &unwrap_parens(expr).kind {
        ExprKind::FunctionCall(fc) => &fc.args,
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => &mc.args,
        ExprKind::StaticMethodCall(smc) => &smc.args,
        _ => return false,
    };
    args.iter().any(|a| {
        a.value.as_ref().is_some_and(|v| {
            matches!(unwrap_parens(v).kind, ExprKind::Closure(_)) || has_closure_argument(v)
        })
    })
}

/// The closure a call target evaluates to: a variable bound to one, or a literal.
fn callee_closure(
    callee: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<std::sync::Arc<crate::flow_state::ClosureSummary>> {
    let callee = unwrap_parens(callee);
    match &callee.kind {
        ExprKind::Variable(name) => ctx
            .closures
            .by_var
            .get(&mir_types::Name::from(name.trim_start_matches('$')))
            .cloned(),
        ExprKind::Closure(_) => ctx.closures.by_span.get(&callee.span.start).cloned(),
        ExprKind::ArrowFunction(af) => Some(
            ctx.closures
                .by_span
                .get(&callee.span.start)
                .cloned()
                .unwrap_or_else(|| std::sync::Arc::new(arrow_summary(af, ctx, db, file))),
        ),
        _ => None,
    }
}

/// The taint a captured variable carries inside a closure: its source taint,
/// and a slot past the closure's parameters standing for the enclosing body's
/// labels, which `captures` records.
pub fn captured_taint(outer: Taint, param_count: usize, captures: &mut Vec<Taint>) -> Taint {
    let slot = param_count + captures.len();
    if outer.labels().is_empty() || slot >= MAX_TRACKED_PARAMS {
        return outer.concrete();
    }
    captures.push(outer.labels());
    outer.concrete() | Taint::param(slot)
}

/// Variables an expression reads, in source order.
struct VariableReads(Vec<mir_types::Name>);

impl php_ast::owned::visitor::OwnedVisitor for VariableReads {
    fn visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
        if let ExprKind::Variable(name) = &expr.kind {
            let name = mir_types::Name::from(name.trim_start_matches('$'));
            if !self.0.contains(&name) {
                self.0.push(name);
            }
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// An arrow function's summary, from its body evaluated with labelled parameters.
pub fn arrow_summary(
    af: &php_ast::owned::ArrowFunctionExpr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> crate::flow_state::ClosureSummary {
    let params: Vec<(mir_types::Name, bool)> = af
        .params
        .iter()
        .map(|p| {
            (
                mir_types::Name::new(p.name.as_deref().unwrap_or("")),
                p.variadic,
            )
        })
        .collect();
    let mut body_ctx = ctx.clone();
    for (name, _) in &params {
        body_ctx.set_var(name.as_str(), mir_types::Type::mixed());
    }
    label_params(&mut body_ctx, params.iter().map(|(n, _)| n.as_str()));
    for (name, taint) in body_ctx.tainted_vars.iter_mut() {
        if !params.iter().any(|(p, _)| p == name) {
            *taint = taint.concrete();
        }
    }
    let mut reads = VariableReads(Vec::new());
    let _ = php_ast::owned::visitor::OwnedVisitor::visit_expr(&mut reads, &af.body);
    let mut captures = Vec::new();
    for name in reads.0 {
        if params.iter().any(|(p, _)| *p == name) {
            continue;
        }
        let taint = captured_taint(ctx.var_taint(name.as_str()), params.len(), &mut captures);
        if !taint.is_empty() {
            body_ctx.tainted_vars.insert(name, taint);
        }
    }
    crate::flow_state::ClosureSummary {
        params,
        summary: TaintSummary {
            returns: expr_taint(&af.body, &body_ctx, db, file),
            ..TaintSummary::default()
        },
        captures,
    }
}

/// Expressions passed for a closure's parameter `index`.
fn closure_param_args<'e>(
    params: &[(mir_types::Name, bool)],
    args: &'e [php_ast::owned::Arg],
    index: usize,
) -> Vec<&'e Expr> {
    let Some((name, variadic)) = params.get(index) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut position = 0;
    for arg in args {
        let Some(value) = &arg.value else {
            position += 1;
            continue;
        };
        match &arg.name {
            Some(arg_name) => {
                if crate::parser::name_to_string_owned(arg_name) == name.as_str() {
                    out.push(value);
                }
            }
            None => {
                if arg.unpack || position == index || (*variadic && position >= index) {
                    out.push(value);
                }
                position += 1;
            }
        }
    }
    out
}

fn closure_call_taint(
    closure: &crate::flow_state::ClosureSummary,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    closure.summary.call_result(|slot| {
        closure.capture(slot).unwrap_or_else(|| {
            closure_param_args(&closure.params, args, slot)
                .into_iter()
                .fold(Taint::NONE, |acc, e| acc | expr_taint(e, ctx, db, file))
        })
    })
}

/// A callback named by a string (`'trim'`) or a first-class callable (`trim(...)`).
fn callback_function_name(callback: &Expr) -> Option<&str> {
    match &unwrap_parens(callback).kind {
        ExprKind::String(name) if !name.contains("::") => Some(name.as_ref()),
        ExprKind::CallableCreate(c) => match &c.kind {
            php_ast::owned::CallableCreateKind::Function(callee) => match &callee.kind {
                ExprKind::Identifier(name) => Some(name.as_ref()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Taint of what calling `callback` returns when it receives `inputs`.
fn callback_result(
    callback: &Expr,
    inputs: &[Taint],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let all = inputs.iter().fold(Taint::NONE, |acc, t| acc | *t);
    let input = |index: usize| inputs.get(index).copied().unwrap_or_default();
    if let Some(closure) = callee_closure(callback, ctx, db, file) {
        return closure
            .summary
            .call_result(|slot| closure.capture(slot).unwrap_or_else(|| input(slot)));
    }
    let Some(name) = callback_function_name(callback) else {
        return all;
    };
    let lower = crate::util::php_ident_lowercase(name.trim_start_matches('\\'));
    if let Some(removed) = sanitized_kinds(&lower) {
        return all.without(removed);
    }
    if let Some(restored) = unescaped_kinds(&lower) {
        return all.restore(restored);
    }
    let resolved = crate::db::resolve_name(db, file, name);
    let Some(func) =
        crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved))
    else {
        return all;
    };
    if func
        .return_type
        .as_deref()
        .is_some_and(|ty| ty.is_void() || is_injection_safe_type(ty))
    {
        return Taint::NONE;
    }
    match CalleeBody::of(func.location.as_ref(), file) {
        CalleeBody::Summarized if !demanding_taint(all, ctx).is_empty() => {
            crate::db::function_taint_summary(db, file, func.fqn.as_ref())
                .map_or(Taint::NONE, |s| s.call_result(input))
        }
        CalleeBody::Summarized => Taint::NONE,
        _ => all,
    }
}

/// Builtins that call a callback on an array's elements: the callback's
/// argument index and, for each of its parameters, the argument it receives.
fn callback_inputs(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
) -> Option<(usize, Vec<usize>)> {
    Some(match fn_name_lower {
        "array_map" => (0, (1..args.len().max(2)).collect()),
        "array_filter" | "array_walk" | "array_walk_recursive" => (1, vec![0]),
        "usort" | "uasort" | "uksort" => (1, vec![0, 0]),
        "array_reduce" => (1, vec![2, 0]),
        "preg_replace_callback" => (1, vec![2]),
        _ => return None,
    })
}

/// Result taint of a builtin that calls a callback on its input.
fn callback_builtin_taint(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let arg_taint = |index: usize| {
        args.get(index)
            .and_then(|a| a.value.as_ref())
            .map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file))
    };
    let (callback_index, input_args) = callback_inputs(fn_name_lower, args)?;
    let inputs: Vec<Taint> = input_args.iter().map(|i| arg_taint(*i)).collect();
    let callback = args.get(callback_index).and_then(|a| a.value.as_ref());
    let called = || {
        callback.map_or(Taint::NONE, |cb| {
            callback_result(cb, &inputs, ctx, db, file)
        })
    };
    Some(match fn_name_lower {
        "array_map" => called(),
        "array_filter" => arg_taint(0),
        "array_reduce" => arg_taint(2) | called(),
        "preg_replace_callback" => arg_taint(2) | called(),
        _ => return None,
    })
}

/// Report data a callback builtin hands to a callback whose body reaches a sink.
pub fn check_callback_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let Some((callback_index, input_args)) = callback_inputs(fn_name_lower, args) else {
        return;
    };
    let Some(callback) = args.get(callback_index).and_then(|a| a.value.as_ref()) else {
        return;
    };
    let closure = callee_closure(callback, ctx, ea.db, &ea.file);
    let param_sinks = match &closure {
        Some(closure) => closure.summary.param_sinks.clone(),
        None => {
            let Some(name) = callback_function_name(callback) else {
                return;
            };
            let resolved = crate::db::resolve_name(ea.db, &ea.file, name);
            crate::db::function_param_sinks(ea.db, &ea.file, &resolved)
                .map(|s| s.param_sinks)
                .unwrap_or_default()
        }
    };
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for sink in &param_sinks {
        if let Some(captured) = closure.as_ref().and_then(|c| c.capture(sink.param)) {
            ea.taint_summary
                .record_param_sinks(captured, sink.kinds, &sink.issue);
            continue;
        }
        let Some(value) = input_args
            .get(sink.param)
            .and_then(|i| args.get(*i))
            .and_then(|a| a.value.as_ref())
        else {
            continue;
        };
        let taint = expr_taint(value, ctx, ea.db, &ea.file);
        ea.taint_summary
            .record_param_sinks(taint, sink.kinds, &sink.issue);
        if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
            emitted.push(sink.issue.clone());
            ea.emit_taint(sink.issue.clone(), span, value, sink.kinds, ctx);
        }
    }
}

/// Report arguments a call of a closure passes to a sink inside it.
pub fn check_closure_call_sinks(
    ea: &mut crate::expr::ExpressionAnalyzer<'_>,
    callee: &Expr,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    span: php_ast::Span,
) {
    let Some(closure) = callee_closure(callee, ctx, ea.db, &ea.file) else {
        return;
    };
    let mut emitted: Vec<mir_issues::IssueKind> = Vec::new();
    for sink in &closure.summary.param_sinks {
        // Source data a capture holds was reported inside the closure's body.
        if let Some(captured) = closure.capture(sink.param) {
            ea.taint_summary
                .record_param_sinks(captured, sink.kinds, &sink.issue);
            continue;
        }
        for value in closure_param_args(&closure.params, args, sink.param) {
            let taint = expr_taint(value, ctx, ea.db, &ea.file);
            ea.taint_summary
                .record_param_sinks(taint, sink.kinds, &sink.issue);
            if taint.concrete().intersects(sink.kinds) && !emitted.contains(&sink.issue) {
                emitted.push(sink.issue.clone());
                ea.emit_taint(sink.issue.clone(), span, value, sink.kinds, ctx);
            }
        }
    }
}

/// Whether a declaration at `location` has a body worth summarizing.
fn is_user_code(location: Option<&mir_types::Location>) -> bool {
    location.is_some_and(|loc| !crate::stubs::is_stub_path(loc.file.as_ref()))
}

/// `…/vendor/<vendor>/<package>` prefix of a Composer dependency's path.
fn vendor_package(path: &str) -> Option<&str> {
    let start = path
        .match_indices("vendor")
        .filter(|(i, _)| *i == 0 || path[..*i].ends_with(['/', '\\']))
        .map(|(i, _)| i + "vendor".len())
        .filter(|end| path[*end..].starts_with(['/', '\\']))
        .last()?;
    let mut separators = path[start..]
        .match_indices(['/', '\\'])
        .map(|(i, _)| start + i);
    separators.next()?;
    separators.next()?;
    Some(separators.next().map_or(path, |end| &path[..end]))
}

/// A callee in another Composer package than `from` is treated like a builtin:
/// its arguments pass through, and its body isn't summarized.
pub fn is_dependency_of(callee_path: &str, from: &str) -> bool {
    vendor_package(callee_path).is_some_and(|package| vendor_package(from) != Some(package))
}

/// Taint of a call to `method_lower` on `class`, or `None` when the method is unknown.
fn method_call_taint(
    class: crate::db::Fqcn<'_>,
    method_lower: &str,
    args: &[php_ast::owned::Arg],
    receiver: Option<&Expr>,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let (owner, m) = crate::db::find_method_respecting_precedence(db, class, method_lower)?;
    if m.is_taint_source {
        return Some(Taint::ALL);
    }
    if !is_user_code(m.location.as_ref()) && m.taint.is_none() {
        return None;
    }
    if m.taint.is_none() {
        let implementations =
            crate::db::taint_sources::method_implementations(db, owner.as_ref(), method_lower);
        if !implementations.is_empty() {
            let taint = implementations
                .iter()
                .fold(Taint::NONE, |acc, implementor| {
                    let here = crate::db::Fqcn::from_str(db, implementor.as_str());
                    acc | method_call_taint(here, method_lower, args, receiver, ctx, db, file)
                        .unwrap_or_default()
                });
            return (!taint.is_empty()).then_some(taint);
        }
    }
    let body = if m.is_abstract || crate::db::is_interface(db, owner.as_ref()) {
        CalleeBody::None
    } else {
        CalleeBody::of(m.location.as_ref(), file)
    };
    let taint = user_call_taint(
        &m.params,
        args,
        m.return_type.as_deref(),
        m.taint.as_deref(),
        body,
        receiver,
        || crate::db::taint_sources::method_returns_param(db, file, owner.as_ref(), method_lower),
        || crate::db::taint_sources::method_returns_prop(db, file, owner.as_ref(), method_lower),
        || crate::db::taint_sources::method_reads_source(db, file, owner.as_ref(), method_lower),
        || crate::db::method_taint_summary(db, file, owner.as_ref(), method_lower),
        ctx,
        db,
        file,
    );
    (!taint.is_empty()).then_some(taint)
}

/// Builtins whose result is request data.
pub fn is_source_function(fn_name_lower: &str) -> bool {
    matches!(
        fn_name_lower,
        "getallheaders" | "apache_request_headers" | "filter_input" | "filter_input_array"
    )
}

/// `file_get_contents('php://input')` and friends read the raw request body.
fn reads_raw_request_body(fn_name_lower: &str, args: &[php_ast::owned::Arg]) -> bool {
    matches!(
        fn_name_lower,
        "file_get_contents" | "fopen" | "file" | "readfile" | "stream_get_contents"
    ) && matches!(
        args.first().and_then(|a| a.value.as_ref()).map(|v| &v.kind),
        Some(ExprKind::String(path))
            if path.eq_ignore_ascii_case("php://input")
    )
}

/// Positions (among the values after the format) a printf-style format
/// interpolates as text; numeric conversions (`%d`, `%.2f`, `%x`) can't carry
/// a payload. `None` when the format is malformed.
fn format_text_positions(format: &str) -> Option<Vec<usize>> {
    let mut positions = Vec::new();
    let mut next = 0;
    let mut chars = format.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '%' {
            continue;
        }
        if chars.peek() == Some(&'%') {
            chars.next();
            continue;
        }
        let mut digits = String::new();
        while let Some(d) = chars.peek().copied().filter(char::is_ascii_digit) {
            digits.push(d);
            chars.next();
        }
        let position = if chars.peek() == Some(&'$') && !digits.is_empty() {
            chars.next();
            digits.parse::<usize>().ok()?.checked_sub(1)?
        } else {
            let position = next;
            next += 1;
            position
        };
        loop {
            match chars.next()? {
                '-' | '+' | ' ' | '0'..='9' | '.' => {}
                '\'' => {
                    chars.next()?;
                }
                's' | 'c' => {
                    positions.push(position);
                    break;
                }
                'b' | 'd' | 'e' | 'E' | 'f' | 'F' | 'g' | 'G' | 'h' | 'H' | 'o' | 'u' | 'x'
                | 'X' => break,
                _ => return None,
            }
        }
    }
    Some(positions)
}

/// The arguments of a printf-family call whose text ends up in the result:
/// the format itself and the values it interpolates with `%s`. `None` when the
/// call isn't printf-style.
pub fn format_text_args<'e>(
    fn_name_lower: &str,
    args: &'e [php_ast::owned::Arg],
) -> Option<Vec<&'e Expr>> {
    let (format_index, values_as_array) = match fn_name_lower {
        "sprintf" | "printf" => (0, false),
        "fprintf" => (1, false),
        "vsprintf" | "vprintf" => (0, true),
        "vfprintf" => (1, true),
        _ => return None,
    };
    if args.iter().any(|a| a.name.is_some() || a.unpack) {
        return None;
    }
    let format = args.get(format_index)?.value.as_ref()?;
    let values = || {
        args.iter()
            .skip(format_index + 1)
            .filter_map(|a| a.value.as_ref())
    };
    let ExprKind::String(literal) = &format.kind else {
        return Some(std::iter::once(format).chain(values()).collect());
    };
    let Some(positions) = format_text_positions(literal) else {
        return Some(std::iter::once(format).chain(values()).collect());
    };
    if values_as_array {
        return Some(if positions.is_empty() {
            Vec::new()
        } else {
            values().collect()
        });
    }
    Some(
        values()
            .enumerate()
            .filter(|(i, _)| positions.contains(i))
            .map(|(_, v)| v)
            .collect(),
    )
}

/// Whether a flags expression includes the constant `flag` (`A | B`).
fn has_flag(expr: &Expr, flag: &str) -> bool {
    match &expr.kind {
        ExprKind::Identifier(name) => name.trim_start_matches('\\') == flag,
        ExprKind::Binary(op) if op.op == php_ast::ast::BinaryOp::BitwiseOr => {
            has_flag(&op.left, flag) || has_flag(&op.right, flag)
        }
        ExprKind::Parenthesized(inner) => has_flag(inner, flag),
        _ => false,
    }
}

/// Taint of a `filter_var()`/`filter_input()` result: validating filters
/// (`FILTER_VALIDATE_INT`, …) leave nothing injectable, escaping ones remove
/// their own kind, everything else passes the input through.
fn filter_call_taint(
    fn_name_lower: &str,
    args: &[php_ast::owned::Arg],
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Taint> {
    let arg = |i: usize| args.get(i).and_then(|a| a.value.as_ref());
    let (input, filter) = match fn_name_lower {
        "filter_var" => (
            arg(0).map_or(Taint::NONE, |v| expr_taint(v, ctx, db, file)),
            arg(1),
        ),
        "filter_input" => (Taint::ALL, arg(2)),
        "filter_input_array" => return Some(Taint::ALL),
        _ => return None,
    };
    let filter_name = match filter.map(|f| &f.kind) {
        Some(ExprKind::Identifier(name)) => name.trim_start_matches('\\').to_ascii_uppercase(),
        _ => return Some(input),
    };
    Some(match filter_name.as_str() {
        "FILTER_VALIDATE_INT"
        | "FILTER_VALIDATE_FLOAT"
        | "FILTER_VALIDATE_BOOLEAN"
        | "FILTER_VALIDATE_BOOL"
        | "FILTER_VALIDATE_IP"
        | "FILTER_VALIDATE_MAC"
        | "FILTER_SANITIZE_NUMBER_INT"
        | "FILTER_SANITIZE_NUMBER_FLOAT" => Taint::NONE,
        "FILTER_SANITIZE_SPECIAL_CHARS" | "FILTER_SANITIZE_FULL_SPECIAL_CHARS" => {
            input.without(Taint::HTML)
        }
        "FILTER_SANITIZE_ENCODED" => input.without(sanitized_kinds("urlencode")?),
        _ => input,
    })
}

/// Taint kinds a sanitizing builtin removes from its arguments' taint.
fn sanitized_kinds(fn_name_lower: &str) -> Option<Taint> {
    Some(match fn_name_lower {
        "htmlspecialchars" | "htmlentities" | "strip_tags" | "e" => Taint::HTML,
        "addslashes"
        | "mysqli_real_escape_string"
        | "mysqli_escape_string"
        | "mysql_real_escape_string"
        | "mysql_escape_string"
        | "pg_escape_string"
        | "pg_escape_literal"
        | "pg_escape_identifier"
        | "pg_escape_bytea"
        | "sqlite_escape_string"
        | "db2_escape_string" => Taint::SQL,
        "escapeshellarg" | "escapeshellcmd" => Taint::SHELL,
        "ldap_escape" => Taint::LDAP,
        // Output is limited to `[A-Za-z0-9%+._~-]`.
        "urlencode" | "rawurlencode" => {
            Taint::HTML
                | Taint::SQL
                | Taint::SHELL
                | Taint::HEADER
                | Taint::COOKIE
                | Taint::LDAP
                | Taint::XPATH
        }
        "md5" | "sha1" | "crc32" | "hash" | "hash_hmac" | "password_hash" | "bin2hex"
        | "dechex" | "decbin" | "decoct" | "number_format" | "uniqid" => Taint::ALL,
        _ => return None,
    })
}

/// Taint kinds a decoding builtin restores on already-tainted input.
fn unescaped_kinds(fn_name_lower: &str) -> Option<Taint> {
    Some(match fn_name_lower {
        "html_entity_decode" | "htmlspecialchars_decode" => Taint::HTML,
        "stripslashes" | "stripcslashes" => Taint::SQL,
        "urldecode" | "rawurldecode" => sanitized_kinds("urlencode")?,
        _ => return None,
    })
}

/// Taint kinds an escaping method of a builtin database class removes.
fn sanitized_method_kinds(
    db: &dyn crate::db::MirDatabase,
    fqcn: &str,
    method_lower: &str,
) -> Option<Taint> {
    let is_a = |base: &str| {
        fqcn.eq_ignore_ascii_case(base) || crate::db::extends_or_implements(db, fqcn, base)
    };
    let escapes = match method_lower {
        "quote" => is_a("PDO"),
        "real_escape_string" | "escape_string" => is_a("mysqli"),
        "escapestring" => is_a("SQLite3"),
        _ => false,
    };
    escapes.then_some(Taint::SQL)
}
