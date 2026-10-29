use super::*;
use crate::ast_ext::ArgExt as _;

/// What a label slot past the parameters stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SlotLabel {
    Prop(mir_types::Name),
    /// The parameter's own value, once proven an object.
    Guard(usize),
}

/// A parameter whose value reaches a sink inside the callee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamSink {
    pub param: usize,
    pub kinds: Taint,
    pub issue: mir_issues::IssueKind,
}

/// Taint of the key names of the array a body returns.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ReturnKeys {
    /// No `return` of a value seen.
    #[default]
    Unseen,
    /// The key names carry exactly this taint, from real sources only.
    Exact(Taint),
    /// The key names may carry anything the returned value does.
    Unknown,
}

impl ReturnKeys {
    pub(super) fn combine(self, other: ReturnKeys) -> ReturnKeys {
        match (self, other) {
            (ReturnKeys::Unseen, x) | (x, ReturnKeys::Unseen) => x,
            (ReturnKeys::Exact(a), ReturnKeys::Exact(b)) => ReturnKeys::Exact(a | b),
            _ => ReturnKeys::Unknown,
        }
    }
}

/// How a function or method moves taint, seen from its call sites.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TaintSummary {
    /// Taint of the return value: the source slot is taint the body reads
    /// itself, each parameter slot the kinds of that argument that flow out.
    pub returns: Taint,
    pub return_keys: ReturnKeys,
    pub param_sinks: Vec<ParamSink>,
    /// Taint written to the class's own properties, keyed like
    /// [`crate::db::taint_sources::prop_key`].
    pub prop_writes: Vec<(mir_types::Name, Taint)>,
    /// Label slots past the parameters that stand for `$this`'s properties.
    pub prop_labels: Vec<(usize, mir_types::Name)>,
    /// `(slot, param)`: label slots past the parameters, each standing for a
    /// parameter's own value once the body proved it an object.
    pub object_guards: Vec<(usize, usize)>,
    /// `(param, taint)`: what a by-ref parameter holds when the body exits.
    pub out_params: Vec<(usize, Taint)>,
    /// Label slots the return value carries only as (part of) the argument
    /// itself; see [`param_parts`].
    pub return_parts: Taint,
    /// Label slots some `return` carries otherwise, while the body is analyzed.
    pub(super) returned_computed: Taint,
    /// Taint of the values a generator body yields; a generator's result stands for them.
    pub(super) yields: Taint,
    /// Properties of `$this` set by a plain assignment while the body is analyzed:
    /// their `prop_writes` entry is what they hold when the body exits or calls a method.
    pub(super) overwritten_props: Vec<mir_types::Name>,
}

impl TaintSummary {
    pub fn is_empty(&self) -> bool {
        self.returns.is_empty()
            && self.param_sinks.is_empty()
            && self.prop_writes.is_empty()
            && self.out_params.is_empty()
    }

    /// Record what the by-ref parameters hold at a point where the body exits.
    pub fn record_out_params(&mut self, ctx: &FlowState) {
        for (slot, name) in ctx.byref_out_slots.iter() {
            let held = ctx.var_taint(name.as_str());
            if held.is_empty() {
                continue;
            }
            match self.out_params.iter_mut().find(|(p, _)| p == slot) {
                Some((_, existing)) => *existing |= held,
                None => self.out_params.push((*slot, held)),
            }
        }
    }

    /// Taint by-ref parameter `param` holds on exit.
    pub fn out_param(&self, param: usize) -> Taint {
        self.out_params
            .iter()
            .filter(|(p, _)| *p == param)
            .fold(Taint::NONE, |acc, (_, t)| acc | *t)
    }

    /// Record a `return` of a value with `taint`, `parts` of whose label slots
    /// are (part of) an argument itself.
    pub fn record_return(&mut self, taint: Taint, parts: Taint) {
        self.returns |= taint;
        self.return_parts |= parts;
        self.returned_computed |= taint.label_slots().difference(parts);
    }

    /// Record the key names of a `return`ed array, `None` when they are not known exactly.
    pub fn record_return_keys(&mut self, keys: Option<Taint>) {
        let keys = match keys {
            Some(taint) if taint.labels().is_empty() => ReturnKeys::Exact(taint),
            _ => ReturnKeys::Unknown,
        };
        self.return_keys = self.return_keys.combine(keys);
    }

    /// Record a `yield` of a key or value with `taint`.
    pub fn record_yield(&mut self, taint: Taint) {
        self.yields |= taint;
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

    pub fn note_prop_overwrite(&mut self, prop: mir_types::Name) {
        if !self.overwritten_props.contains(&prop) {
            self.overwritten_props.push(prop);
        }
    }

    /// Record what the overwritten properties hold now, at a point where they can be read.
    pub fn record_overwritten_props(&mut self, ctx: &FlowState) {
        for i in 0..self.overwritten_props.len() {
            let prop = self.overwritten_props[i];
            // The property's own label stands for its old value, which a write doesn't add.
            let held = self
                .prop_labels
                .iter()
                .filter(|(_, label)| *label == prop)
                .fold(ctx.prop_taint("this", prop.as_str()), |held, (slot, _)| {
                    held.without_label(*slot)
                });
            self.record_prop_write(prop, held);
        }
    }

    /// Combine with another body's summary, for a call that may run either.
    pub fn merge(&mut self, mut other: TaintSummary) {
        // Parameters keep their slots, which line up with a call's arguments;
        // a guard's parameter stays free for its flows to fall back to.
        let mut reserved = [false; MAX_TRACKED_PARAMS];
        for summary in [&*self, &other] {
            for (slot, reserve) in reserved.iter_mut().enumerate() {
                *reserve |= summary.reads_slot(slot) && summary.slot_label(slot).is_none();
            }
            for (_, param) in &summary.object_guards {
                reserved[*param] = true;
            }
        }
        let mut placed: Vec<(usize, SlotLabel)> = Vec::new();
        let mut moves: [Vec<(usize, Option<usize>)>; 2] = Default::default();
        for (side, summary) in [&*self, &other].into_iter().enumerate() {
            for (slot, label) in summary.slot_labels() {
                let free = |s: usize| !reserved[s] && !placed.iter().any(|(p, _)| *p == s);
                let target = placed
                    .iter()
                    .find(|(_, l)| *l == label)
                    .map(|(s, _)| *s)
                    .or_else(|| free(slot).then_some(slot))
                    .or_else(|| (0..MAX_TRACKED_PARAMS).find(|s| free(*s)));
                if let Some(target) = target.filter(|t| !placed.contains(&(*t, label))) {
                    placed.push((target, label));
                }
                moves[side].push((slot, target));
            }
        }
        let [self_moves, other_moves] = moves;
        self.move_labels(&self_moves);
        other.move_labels(&other_moves);
        self.prop_labels = placed
            .iter()
            .filter_map(|(slot, label)| match label {
                SlotLabel::Prop(prop) => Some((*slot, *prop)),
                SlotLabel::Guard(_) => None,
            })
            .collect();
        self.object_guards = placed
            .iter()
            .filter_map(|(slot, label)| match label {
                SlotLabel::Guard(param) => Some((*slot, *param)),
                SlotLabel::Prop(_) => None,
            })
            .collect();
        let computed = |s: &TaintSummary| s.returns.label_slots().difference(s.return_parts);
        self.return_parts =
            (self.return_parts | other.return_parts).difference(computed(self) | computed(&other));
        self.returns |= other.returns;
        self.return_keys = self.return_keys.combine(other.return_keys);
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
        for (param, taint) in other.out_params {
            match self.out_params.iter_mut().find(|(p, _)| *p == param) {
                Some((_, existing)) => *existing |= taint,
                None => self.out_params.push((param, taint)),
            }
        }
    }

    /// Source taint written to property `key` whatever the arguments.
    pub fn prop_write(&self, key: mir_types::Name) -> Taint {
        self.prop_writes
            .iter()
            .filter(|(k, _)| *k == key)
            .fold(Taint::NONE, |acc, (_, t)| acc | t.concrete())
    }

    /// Move each label slot to its new slot; a guard with none falls back to its
    /// parameter, and a property's flows are lost.
    fn move_labels(&mut self, moves: &[(usize, Option<usize>)]) {
        for (slot, target) in moves {
            if target.is_none() {
                match self.slot_label(*slot) {
                    Some(SlotLabel::Guard(_)) => self.unguard(*slot),
                    _ => self.drop_label(*slot),
                }
            }
        }
        let target = |slot: usize| match moves.iter().find(|(s, _)| *s == slot) {
            Some((_, target)) => *target,
            None => Some(slot),
        };
        let relabel = |taint: Taint| {
            (0..MAX_TRACKED_PARAMS).fold(taint.concrete(), |acc, slot| match target(slot) {
                Some(to) => acc | Taint::label(to, taint.param_kinds(slot)),
                None => acc,
            })
        };
        self.returns = relabel(self.returns);
        self.return_parts = relabel(self.return_parts);
        for taint in self
            .prop_writes
            .iter_mut()
            .map(|(_, t)| t)
            .chain(self.out_params.iter_mut().map(|(_, t)| t))
        {
            *taint = relabel(*taint);
        }
        for sink in std::mem::take(&mut self.param_sinks) {
            let Some(param) = target(sink.param) else {
                continue;
            };
            match self
                .param_sinks
                .iter_mut()
                .find(|s| s.param == param && s.issue == sink.issue)
            {
                Some(existing) => existing.kinds |= sink.kinds,
                None => self.param_sinks.push(ParamSink { param, ..sink }),
            }
        }
    }

    /// The property or guard label slot `slot` stands for.
    fn slot_label(&self, slot: usize) -> Option<SlotLabel> {
        self.prop_label(slot).map(SlotLabel::Prop).or_else(|| {
            self.object_guards
                .iter()
                .find(|(s, _)| *s == slot)
                .map(|(_, param)| SlotLabel::Guard(*param))
        })
    }

    fn slot_labels(&self) -> Vec<(usize, SlotLabel)> {
        let props = self
            .prop_labels
            .iter()
            .map(|(slot, prop)| (*slot, SlotLabel::Prop(*prop)));
        let guards = self
            .object_guards
            .iter()
            .map(|(slot, param)| (*slot, SlotLabel::Guard(*param)));
        props.chain(guards).collect()
    }

    /// Whether some flow of this body comes from label slot `slot`.
    fn reads_slot(&self, slot: usize) -> bool {
        let label = Taint::param(slot);
        self.returns.intersects(label)
            || self
                .prop_writes
                .iter()
                .map(|(_, taint)| taint)
                .chain(self.out_params.iter().map(|(_, taint)| taint))
                .any(|taint| taint.intersects(label))
            || self.param_sinks.iter().any(|sink| sink.param == slot)
    }

    /// Let what flows from object guard slot `slot` flow from its parameter
    /// whatever the argument holds.
    fn unguard(&mut self, slot: usize) {
        let Some(index) = self.object_guards.iter().position(|(s, _)| *s == slot) else {
            return;
        };
        let (_, param) = self.object_guards.remove(index);
        let move_label =
            |taint: Taint| taint.without_label(slot) | Taint::label(param, taint.param_kinds(slot));
        self.returns = move_label(self.returns);
        for taint in self
            .prop_writes
            .iter_mut()
            .map(|(_, t)| t)
            .chain(self.out_params.iter_mut().map(|(_, t)| t))
        {
            *taint = move_label(*taint);
        }
        for sink in std::mem::take(&mut self.param_sinks) {
            let param = if sink.param == slot {
                param
            } else {
                sink.param
            };
            match self
                .param_sinks
                .iter_mut()
                .find(|s| s.param == param && s.issue == sink.issue)
            {
                Some(existing) => existing.kinds |= sink.kinds,
                None => self.param_sinks.push(ParamSink { param, ..sink }),
            }
        }
    }

    /// Forget what flows from label slot `slot`.
    fn drop_label(&mut self, slot: usize) {
        self.returns = self.returns.without_label(slot);
        self.param_sinks.retain(|sink| sink.param != slot);
        for taint in self
            .prop_writes
            .iter_mut()
            .map(|(_, t)| t)
            .chain(self.out_params.iter_mut().map(|(_, t)| t))
        {
            *taint = taint.without_label(slot);
        }
    }

    /// Taint flowing into label slot `slot` at a call: a property of the
    /// receiver, or the arguments bound to that parameter.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn slot_taint(
        &self,
        slot: usize,
        receiver: Option<&Expr>,
        params: &[mir_codebase::DeclaredParam],
        args: &[php_ast::owned::Arg],
        ctx: &FlowState,
        db: &dyn crate::db::MirDatabase,
        file: &str,
    ) -> Taint {
        let guarded = self.is_object_guard(slot);
        match self.prop_label(slot) {
            Some(prop) => receiver_prop_taint(receiver, prop, ctx, db, file),
            None => self
                .slot_args(slot, params, args, &[], ctx, db, file)
                .into_iter()
                .fold(Taint::NONE, |acc, e| {
                    acc | if guarded {
                        object_arg_taint(e, ctx, db, file)
                    } else {
                        value_taint(e, ctx, db, file)
                    }
                }),
        }
    }

    /// Arguments flowing into label slot `slot` at a call; an object guard's
    /// skip those that can't hold an object. `arg_types` lines up with `args`
    /// when the call site knows them.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn slot_args<'e>(
        &self,
        slot: usize,
        params: &[mir_codebase::DeclaredParam],
        args: &'e [php_ast::owned::Arg],
        arg_types: &[mir_types::Type],
        ctx: &FlowState,
        db: &dyn crate::db::MirDatabase,
        file: &str,
    ) -> Vec<&'e Expr> {
        if !self.is_object_guard(slot) {
            return param_arg_exprs(params, args, slot);
        }
        let typed = arg_types.len() == args.len() && !args.iter().any(|a| a.unpack());
        self.guarded_params(slot)
            .flat_map(|param| param_arg_exprs(params, args, param))
            .filter(|value| {
                let ty = typed
                    .then(|| {
                        args.iter()
                            .position(|a| a.value().is_some_and(|v| std::ptr::eq(v, *value)))
                    })
                    .flatten()
                    .and_then(|index| arg_types.get(index));
                ty.is_none_or(type_may_hold_object) && value_may_hold_object(value, ctx, db, file)
            })
            .collect()
    }

    pub fn is_object_guard(&self, slot: usize) -> bool {
        self.object_guards.iter().any(|(s, _)| *s == slot)
    }

    /// The parameters whose value label slot `slot` stands for once proven an object.
    fn guarded_params(&self, slot: usize) -> impl Iterator<Item = usize> + '_ {
        self.object_guards
            .iter()
            .filter(move |(s, _)| *s == slot)
            .map(|(_, param)| *param)
    }

    /// The parameters label slot `slot` reads, whatever their arguments hold.
    pub fn slot_params(&self, slot: usize) -> Vec<usize> {
        if self.is_object_guard(slot) {
            self.guarded_params(slot).collect()
        } else {
            vec![slot]
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
        resolve_labels(self.returns, |slot| {
            self.slot_params(slot)
                .into_iter()
                .fold(Taint::NONE, |acc, param| acc | arg_taint(param))
        })
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
pub fn finish_summary(summary: &mut TaintSummary, ctx: &FlowState) -> TaintSummary {
    if !ctx.diverges {
        summary.record_overwritten_props(ctx);
        summary.record_out_params(ctx);
    }
    let mut summary = std::mem::take(summary);
    summary.overwritten_props.clear();
    if ctx.is_generator {
        summary.return_keys = ReturnKeys::Unknown;
        summary.returns = std::mem::take(&mut summary.yields);
        summary.returned_computed = summary.returns.label_slots();
        summary.return_parts = Taint::NONE;
    }
    summary.yields = Taint::NONE;
    summary.return_parts = summary
        .return_parts
        .intersection(summary.returns.label_slots())
        .difference(std::mem::take(&mut summary.returned_computed));
    let flows = summary
        .prop_writes
        .iter()
        .map(|(_, taint)| taint)
        .chain(summary.out_params.iter().map(|(_, taint)| taint))
        .fold(summary.returns, |acc, taint| acc | *taint);
    summary.object_guards = ctx
        .object_guards()
        .into_iter()
        .filter(|(_, slot)| {
            flows.intersects(Taint::param(*slot))
                || summary.param_sinks.iter().any(|sink| sink.param == *slot)
        })
        .map(|(param, slot)| (slot, param))
        .collect();
    summary
}

/// Label each parameter so the body's summary can tell which arguments flow where.
pub fn seed_param_labels(
    ctx: &mut FlowState,
    params: &[mir_codebase::DeclaredParam],
    summarize: bool,
) {
    if summarize {
        ctx.summarizing_taint = true;
        let named = named_params(params);
        label_params(ctx, named.iter().map(|p| p.name.as_ref()));
        ctx.byref_out_slots = named
            .iter()
            .take(MAX_TRACKED_PARAMS)
            .enumerate()
            .filter(|(_, p)| p.is_byref && !p.is_variadic)
            .map(|(slot, p)| (slot, mir_types::Name::from(p.name.as_ref())))
            .collect();
    }
}

/// The parameters a body can read; the variadic standing for what
/// `func_get_args()` sees has no name.
pub fn named_params(params: &[mir_codebase::DeclaredParam]) -> &[mir_codebase::DeclaredParam] {
    match params.split_last() {
        Some((last, named)) if last.name.as_ref() == "..." => named,
        _ => params,
    }
}

/// Label the properties of `$this` a method body reads, itself or through the
/// methods it calls on `$this`, with the slots its parameters and their object
/// guards leave free, so its summary can tell which of them flow where.
pub fn seed_prop_labels(
    ctx: &mut FlowState,
    params: &[mir_codebase::DeclaredParam],
    body: &[php_ast::owned::Stmt],
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Vec<(usize, mir_types::Name)> {
    let params = named_params(params);
    let param_count = params.len();
    if !ctx.summarizing_taint || param_count >= MAX_TRACKED_PARAMS {
        return Vec::new();
    }
    let mut reads = ThisPropReads::default();
    for stmt in body {
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut reads, stmt);
    }
    let mut props = std::mem::take(&mut reads.props);
    // Parameters the body proves objects keep their guard slots ahead of properties.
    let limit = (MAX_TRACKED_PARAMS - param_count)
        .saturating_sub(tested_param_count(params, body, ctx, db, file));
    for (receiver, method) in &reads.calls {
        if props.len() >= limit {
            break;
        }
        let class = match receiver {
            OwnCallReceiver::Own => ctx.self_fqcn.as_deref(),
            OwnCallReceiver::Parent => ctx.parent_fqcn.as_deref(),
        };
        for prop in class
            .map(|class| called_method_props(db, file, class, method))
            .unwrap_or_default()
        {
            if !props.contains(&prop) {
                props.push(prop);
            }
        }
    }
    props.truncate(limit);
    let labels: Vec<(usize, mir_types::Name)> =
        (param_count..MAX_TRACKED_PARAMS).zip(props).collect();
    for (slot, prop) in &labels {
        ctx.set_prop_taint("this", prop.as_str(), Taint::param(*slot));
    }
    labels
}

/// Properties of `$this` the method `class::method` reads on a path to a sink
/// or into its result.
fn called_method_props(
    db: &dyn crate::db::MirDatabase,
    file: &str,
    class: &str,
    method: &str,
) -> Vec<mir_types::Name> {
    let Some((owner, _)) = crate::db::find_method_respecting_precedence(
        db,
        crate::db::Fqcn::from_str(db, class),
        method,
    ) else {
        return Vec::new();
    };
    if !(crate::db::taint_sources::method_may_reach_sink(db, file, &owner, method)
        || crate::db::taint_sources::method_returns_prop(db, file, &owner, method))
    {
        return Vec::new();
    }
    crate::db::method_taint_summary(db, file, &owner, method)
        .map(|summary| {
            summary
                .prop_labels
                .into_iter()
                .map(|(_, prop)| prop)
                .collect()
        })
        .unwrap_or_default()
}
