use super::helpers::{
    as_concat_str, extract_simple_var, extract_string_from_expr, infer_arithmetic, infer_div,
    infer_int_range_arithmetic, is_non_empty_when_concat, is_property_type_coercion,
    property_assign_compatible, type_refs_any_template, widen_array_as_list,
    widen_array_with_value_and_key,
};
use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::{AssignOp, BinaryOp};
use php_ast::owned::{AssignExpr, Expr, ExprKind};
use php_ast::Span;
use rustc_hash::{FxHashMap, FxHashSet};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_assign(
        &mut self,
        a: &AssignExpr,
        expr_span: Span,
        ctx: &mut FlowState,
    ) -> Type {
        let rhs_tainted = crate::taint::is_expr_tainted(&a.value, ctx);
        // Snapshot which variables were already in consumed_write_locs before
        // analyzing the RHS. When the LHS target variable is consumed DURING RHS
        // analysis (e.g. `$x = f($x)`) the new write to `$x` must be re-armed so it
        // can be independently detected as dead — this mirrors the pre-existing re-arm
        // logic. But variables consumed BEFORE the RHS (carry-forward from a prior
        // loop iteration) must NOT be re-armed, to prevent false "unused" reports on
        // patterns like `foreach (...) { use($prev); $prev = $item; }`.
        let target_var_name: Option<String> = match &a.target.kind {
            ExprKind::Variable(v) => Some(v.trim_start_matches('$').to_string()),
            _ => None,
        };
        let pre_rhs_consumed_count = target_var_name.as_deref().map(|name| {
            let sym = mir_types::Name::from(name);
            ctx.consumed_write_locs
                .iter()
                .filter(|(n, _)| *n == sym)
                .count()
        });
        let rhs_ty = self.analyze(&a.value, ctx);
        if rhs_ty.is_never() {
            return rhs_ty;
        }
        match a.op {
            AssignOp::Assign => {
                self.assign_to_target(&a.target, rhs_ty.clone(), ctx, expr_span);
                // If the target variable was consumed during RHS analysis (e.g. `$x = f($x)`),
                // re-arm the new write location so it is treated as a fresh pending write.
                // This allows subsequent iterations to detect it as dead if never read.
                if let (Some(name), Some(pre_count)) = (&target_var_name, pre_rhs_consumed_count) {
                    let sym = mir_types::Name::from(name.as_str());
                    let post_count = ctx
                        .consumed_write_locs
                        .iter()
                        .filter(|(n, _)| *n == sym)
                        .count();
                    if post_count > pre_count {
                        // Target was freshly consumed during RHS — re-arm the new write.
                        if let Some(locs) = ctx.last_write_locs.get(&sym).cloned() {
                            for loc in locs {
                                ctx.consumed_write_locs.remove(&(sym, loc));
                            }
                        }
                    }
                }
                match &a.target.kind {
                    ExprKind::Variable(name) => {
                        if rhs_tainted {
                            ctx.taint_var(name.as_ref());
                        }
                    }
                    ExprKind::PropertyAccess(pa) => {
                        if let ExprKind::Variable(obj_var) = &pa.object.kind {
                            if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                                let obj_var = obj_var.trim_start_matches('$');
                                if rhs_tainted {
                                    ctx.taint_prop(obj_var, &prop_name);
                                } else {
                                    // Overwritten with a proven-clean value —
                                    // don't let stale taint survive.
                                    ctx.clear_prop_taint(obj_var, &prop_name);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                rhs_ty
            }
            AssignOp::Concat => {
                if let Some(var_name) = extract_simple_var(&a.target) {
                    // `.=` reads the LHS before writing — mark the old write consumed.
                    ctx.mark_consumed(&var_name);
                    let lhs_ty = ctx.get_var(&var_name);
                    let result_ty = if let (Some(l), Some(r)) =
                        (as_concat_str(&lhs_ty), as_concat_str(&rhs_ty))
                    {
                        let combined = format!("{l}{r}");
                        if combined.len() <= 1000 {
                            Type::single(Atomic::TLiteralString(combined.into()))
                        } else {
                            Type::single(Atomic::TNonEmptyString)
                        }
                    } else if is_non_empty_when_concat(&lhs_ty) || is_non_empty_when_concat(&rhs_ty)
                    {
                        Type::single(Atomic::TNonEmptyString)
                    } else {
                        Type::single(Atomic::TString)
                    };
                    ctx.set_var(&var_name, result_ty.clone());
                    let (line, col_start) = self.offset_to_line_col(a.target.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(a.target.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                    result_ty
                } else {
                    // A non-variable target (`$this->log .= 'x'`, `$arr[$k] .= 'x'`)
                    // must still be analyzed like the arithmetic compound ops below —
                    // otherwise the target's own reference recording/existence checks
                    // never run, and the concatenated type is never written back,
                    // leaving the tracked type stale.
                    let lhs_ty = self.analyze(&a.target, ctx);
                    let result_ty = if let (Some(l), Some(r)) =
                        (as_concat_str(&lhs_ty), as_concat_str(&rhs_ty))
                    {
                        let combined = format!("{l}{r}");
                        if combined.len() <= 1000 {
                            Type::single(Atomic::TLiteralString(combined.into()))
                        } else {
                            Type::single(Atomic::TNonEmptyString)
                        }
                    } else if is_non_empty_when_concat(&lhs_ty) || is_non_empty_when_concat(&rhs_ty)
                    {
                        Type::single(Atomic::TNonEmptyString)
                    } else {
                        Type::single(Atomic::TString)
                    };
                    self.assign_to_target(&a.target, result_ty.clone(), ctx, expr_span);
                    result_ty
                }
            }
            AssignOp::Plus
            | AssignOp::Minus
            | AssignOp::Mul
            | AssignOp::Div
            | AssignOp::Mod
            | AssignOp::Pow => {
                // Capture count before LHS analysis: `$a += $i` reads $a (consuming its prior
                // write) then writes a fresh $a. Re-arm the new write so it is independently
                // trackable as a dead write — same logic as AssignOp::Assign.
                let pre_lhs_consumed_count = target_var_name.as_deref().map(|name| {
                    let sym = mir_types::Name::from(name);
                    ctx.consumed_write_locs
                        .iter()
                        .filter(|(n, _)| *n == sym)
                        .count()
                });
                let lhs_ty = self.analyze(&a.target, ctx);
                let range_op = match a.op {
                    AssignOp::Plus => Some(BinaryOp::Add),
                    AssignOp::Minus => Some(BinaryOp::Sub),
                    AssignOp::Div => Some(BinaryOp::Div),
                    _ => None,
                };
                let range_result =
                    range_op.and_then(|op| infer_int_range_arithmetic(&lhs_ty, &rhs_ty, op));
                let result_ty = range_result.unwrap_or_else(|| {
                    if a.op == AssignOp::Div {
                        infer_div(&lhs_ty, &rhs_ty)
                    } else {
                        infer_arithmetic(&lhs_ty, &rhs_ty)
                    }
                });
                self.assign_to_target(&a.target, result_ty.clone(), ctx, expr_span);
                if let (Some(name), Some(pre_count)) = (&target_var_name, pre_lhs_consumed_count) {
                    let sym = mir_types::Name::from(name.as_str());
                    let post_count = ctx
                        .consumed_write_locs
                        .iter()
                        .filter(|(n, _)| *n == sym)
                        .count();
                    if post_count > pre_count {
                        if let Some(locs) = ctx.last_write_locs.get(&sym).cloned() {
                            for loc in locs {
                                ctx.consumed_write_locs.remove(&(sym, loc));
                            }
                        }
                    }
                }
                result_ty
            }
            AssignOp::Coalesce => {
                // `$x ??= 'y'` on an undefined `$x` is valid PHP (treated as if `$x`
                // were null) and afterwards `$x` is exactly the RHS type — not a union
                // with the `mixed` that an undefined-variable read would otherwise
                // produce.
                let is_undefined_var =
                    extract_simple_var(&a.target).is_some_and(|name| !ctx.var_is_defined(&name));
                let lhs_ty = self.with_existence_check(|ea| ea.analyze(&a.target, ctx));
                let merged = if is_undefined_var {
                    rhs_ty.clone()
                } else {
                    Type::merge(&lhs_ty.remove_null(), &rhs_ty)
                };
                // Route through assign_to_target (not just the simple-variable case) so
                // property/array targets are also narrowed — e.g. `$this->x ??= 'y'`
                // should leave $this->x non-null afterwards, not just plain `$x ??= 'y'`.
                self.assign_to_target(&a.target, merged.clone(), ctx, expr_span);
                merged
            }
            _ => {
                if let Some(var_name) = extract_simple_var(&a.target) {
                    // Compound assignment reads the LHS before writing — mark old write consumed.
                    ctx.mark_consumed(&var_name);
                    ctx.set_var(&var_name, Type::mixed());
                    let (line, col_start) = self.offset_to_line_col(a.target.span.start);
                    let (line_end, col_end) = self.offset_to_line_col(a.target.span.end);
                    ctx.record_var_location(&var_name, line, col_start, line_end, col_end);
                }
                Type::mixed()
            }
        }
    }

    pub(crate) fn assign_to_target(
        &mut self,
        target: &Expr,
        ty: Type,
        ctx: &mut FlowState,
        span: Span,
    ) {
        match &target.kind {
            ExprKind::Variable(name) => {
                let name_str = name.trim_start_matches('$').to_string();
                let name_sym = mir_types::Name::from(name_str.as_str());
                // Assigning to $this is not allowed
                if name_str == "this" {
                    self.emit(
                        IssueKind::InvalidScope {
                            in_class: ctx.self_fqcn.is_some(),
                        },
                        Severity::Error,
                        span,
                    );
                }
                if ty.is_mixed_not_template() && name_str != "this" {
                    self.emit(
                        IssueKind::MixedAssignment {
                            var: name_str.clone(),
                        },
                        Severity::Info,
                        span,
                    );
                }
                ctx.set_var(&name_str, ty);
                let (line, col_start) = self.offset_to_line_col(target.span.start);
                let (line_end, col_end) = self.offset_to_line_col(target.span.end);
                if ctx.byref_param_names.contains(&name_sym) {
                    // Byref/global write: mark as read (externally observable) and clear
                    // any pending dead-write entry rather than creating a new one.
                    ctx.read_vars.insert(name_sym);
                    ctx.mark_consumed(&name_str);
                } else {
                    ctx.record_var_location(&name_str, line, col_start, line_end, col_end);
                }
            }
            ExprKind::Array(elements) => {
                let has_non_array = ty.contains(|a| matches!(a, Atomic::TFalse | Atomic::TNull));
                let has_array = ty.contains(|a| {
                    matches!(
                        a,
                        Atomic::TArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TNonEmptyList { .. }
                            | Atomic::TKeyedArray { .. }
                    )
                });
                if has_non_array && has_array {
                    self.emit(
                        IssueKind::PossiblyInvalidArrayOffset {
                            expected: "array".to_string(),
                            actual: format!("{ty}"),
                        },
                        Severity::Warning,
                        span,
                    );
                }
                let fallback_value_ty: Type = ty
                    .types
                    .iter()
                    .find_map(|a| match a {
                        Atomic::TArray { value, .. }
                        | Atomic::TList { value }
                        | Atomic::TNonEmptyArray { value, .. }
                        | Atomic::TNonEmptyList { value } => Some(*value.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(Type::mixed);
                // Destructuring a shape-typed source (`['a' => $a] = $arr` or
                // `[$a, $b] = $arr` against `array{0: int, 1: string}`) should
                // resolve each target's type from the matching per-key
                // property instead of always falling back to `mixed` — the
                // fallback above only covers the plain `TArray`/`TList` shapes.
                let mut next_int_key: i64 = 0;
                for elem in elements.iter() {
                    let key: Option<mir_types::atomic::ArrayKey> = match &elem.key {
                        Some(k) => match &k.kind {
                            ExprKind::String(s) => {
                                Some(match super::helpers::canonical_int_array_key(s) {
                                    Some(i) => mir_types::atomic::ArrayKey::Int(i),
                                    None => mir_types::atomic::ArrayKey::String(
                                        std::sync::Arc::from(s.as_ref()),
                                    ),
                                })
                            }
                            ExprKind::Int(i) => Some(mir_types::atomic::ArrayKey::Int(*i)),
                            _ => None,
                        },
                        None => Some(mir_types::atomic::ArrayKey::Int(next_int_key)),
                    };
                    if elem.key.is_none() {
                        next_int_key += 1;
                    }
                    let elem_ty = key
                        .as_ref()
                        .and_then(|k| {
                            let mut result = Type::empty();
                            let mut found_any = false;
                            for atomic in &ty.types {
                                if let Atomic::TKeyedArray { properties, .. } = atomic {
                                    if let Some(prop) = properties.get(k) {
                                        // Same undefined-offset-then-null semantics as
                                        // plain array access (`expr/arrays.rs`) — an
                                        // optional key may be absent at runtime, so the
                                        // destructured value must include null.
                                        if prop.optional {
                                            let mut widened = prop.ty.clone();
                                            widened.add_type(Atomic::TNull);
                                            result.merge_with(&widened);
                                        } else {
                                            result.merge_with(&prop.ty);
                                        }
                                        found_any = true;
                                    }
                                }
                            }
                            found_any.then_some(result)
                        })
                        .unwrap_or_else(|| fallback_value_ty.clone());
                    self.assign_to_target(&elem.value, elem_ty, ctx, span);
                }
            }
            ExprKind::PropertyAccess(pa) => {
                // Purity check: assigning to a parameter's property in a @pure function.
                if ctx.is_in_pure_fn {
                    if let ExprKind::Variable(recv_name) = &pa.object.kind {
                        let recv_stripped = recv_name.trim_start_matches('$');
                        if ctx
                            .param_names
                            .contains(&mir_types::Name::from(recv_stripped))
                        {
                            if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                                self.emit(
                                    IssueKind::ImpurePropertyAssignment {
                                        property: prop_name,
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                        }
                    }
                }
                // External-mutation-free check: assigning to a parameter's property in
                // a @psalm-external-mutation-free method is forbidden.
                if ctx.is_in_external_mutation_free_method {
                    if let ExprKind::Variable(recv_name) = &pa.object.kind {
                        let recv_stripped = recv_name.trim_start_matches('$');
                        if recv_stripped != "this"
                            && ctx
                                .param_names
                                .contains(&mir_types::Name::from(recv_stripped))
                        {
                            if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                                self.emit(
                                    IssueKind::ImpurePropertyAssignment {
                                        property: prop_name,
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                        }
                    }
                }
                // Immutability check: assigning to $this->prop in a @psalm-immutable class.
                if ctx.is_in_immutable_method {
                    if let ExprKind::Variable(recv_name) = &pa.object.kind {
                        if recv_name.trim_start_matches('$') == "this" {
                            if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                                self.emit(
                                    IssueKind::ImmutablePropertyModification {
                                        property: prop_name,
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                        }
                    }
                }
                let obj_ty = self.analyze(&pa.object, ctx);
                let prop_name_opt = extract_string_from_expr(&pa.property);
                if prop_name_opt.is_none() {
                    self.analyze(&pa.property, ctx);
                }
                if obj_ty.is_mixed() {
                    if let Some(ref prop_name) = prop_name_opt {
                        self.emit(
                            IssueKind::MixedPropertyAssignment {
                                property: prop_name.clone(),
                            },
                            Severity::Info,
                            span,
                        );
                    }
                } else if let Some(prop_name) = prop_name_opt {
                    for atomic in &obj_ty.types {
                        if let Atomic::TNamedObject { fqcn, type_params } = atomic {
                            // Check NoInterfaceProperties for sealed interfaces.
                            if let Some(crate::db::ClassLike::Interface(iface)) =
                                crate::db::find_class_like(
                                    self.db,
                                    crate::db::Fqcn::from_str(self.db, fqcn.as_ref()),
                                )
                            {
                                if iface.seal_properties
                                    && !iface.own_properties.contains_key(prop_name.as_str())
                                {
                                    self.emit(
                                        IssueKind::NoInterfaceProperties {
                                            property: prop_name.clone(),
                                        },
                                        Severity::Info,
                                        span,
                                    );
                                }
                                continue;
                            }
                            let db = self.db;
                            let prop_found = crate::db::find_property_in_chain(
                                db,
                                crate::db::Fqcn::new(db, *fqcn),
                                &prop_name,
                            );
                            let prop_declaring_class =
                                prop_found.as_ref().map(|(cls, _)| cls.clone());
                            let prop_def = prop_found.map(|(_, p)| p);
                            let prop_owner = prop_declaring_class
                                .clone()
                                .unwrap_or_else(|| std::sync::Arc::from(fqcn.as_ref()));
                            // Without this, hover/go-to-definition on the property name
                            // worked on the read side (analyze_property_access) but not
                            // on a plain-assignment write target ($this->prop = ...).
                            self.record_symbol(
                                pa.property.span,
                                crate::symbol::ReferenceKind::PropertyAccess {
                                    class: prop_owner.clone(),
                                    property: std::sync::Arc::from(prop_name.as_str()),
                                },
                                prop_def
                                    .as_ref()
                                    .and_then(|p| p.ty.as_deref().cloned())
                                    .unwrap_or_else(|| ty.clone()),
                            );
                            // Without this, find-all-references on a property only found
                            // reads ($this->prop) — write targets ($this->prop = ...) were
                            // invisible, unlike the read path which also calls record_ref.
                            self.record_ref(
                                std::sync::Arc::from(format!(
                                    "prop:{}::{}",
                                    prop_owner, prop_name
                                )),
                                pa.property.span,
                            );
                            // Emit DeprecatedProperty if the property is deprecated
                            if let Some(ref p) = prop_def {
                                if let Some(msg) = &p.deprecated {
                                    self.emit(
                                        IssueKind::DeprecatedProperty {
                                            class: fqcn.to_string(),
                                            property: prop_name.clone(),
                                            message: Some(msg.clone()).filter(|m| !m.is_empty()),
                                        },
                                        Severity::Info,
                                        span,
                                    );
                                }
                            }
                            let prop_info: Option<(bool, Option<Type>, bool, bool)> =
                                prop_def.map(|p| {
                                    (
                                        p.is_readonly,
                                        p.ty.as_deref().cloned(),
                                        p.has_native_type,
                                        p.has_native_readonly,
                                    )
                                });
                            if let Some((
                                is_readonly,
                                prop_ty,
                                prop_has_native_type,
                                has_native_readonly,
                            )) = prop_info
                            {
                                // PHP 8.1: native readonly (keyword) properties may be initialized
                                // from any method of the declaring class, not just the constructor.
                                // @readonly docblock annotations are advisory and do not get this
                                // exemption. A trait-contributed property counts as part of the
                                // *consuming* class's own scope (PHP copy-paste semantics), so this
                                // checks own composition rather than comparing declaring-class
                                // strings — `find_property_in_chain` reports a trait's own FQCN as
                                // the "declaring class", which would otherwise never match self_fqcn.
                                let in_declaring_scope = ctx.self_fqcn.as_deref().is_some_and(
                                    |self_cls| {
                                        self_cls.eq_ignore_ascii_case(fqcn.as_ref())
                                            && crate::db::property_in_own_composition(
                                                self.db,
                                                crate::db::Fqcn::new(self.db, *fqcn),
                                                &prop_name,
                                            )
                                    },
                                );
                                if is_readonly
                                    && !(ctx.inside_constructor && in_declaring_scope)
                                    && !(has_native_readonly && in_declaring_scope)
                                {
                                    self.emit(
                                        IssueKind::ReadonlyPropertyAssignment {
                                            class: prop_owner.to_string(),
                                            property: prop_name.clone(),
                                        },
                                        Severity::Error,
                                        span,
                                    );
                                }
                                if let Some(prop_ty) = &prop_ty {
                                    // `is_mixed_not_template` (not `is_mixed`): a bare
                                    // `@template T` property type reports `is_mixed() ==
                                    // true` (unconstrained templates default to a `mixed`
                                    // bound), which would skip this check for every generic
                                    // property before its template arg is even considered.
                                    if !prop_ty.is_mixed_not_template()
                                        && !ty.is_mixed_not_template()
                                    {
                                        // Collect all template param names in scope: class-level
                                        // (from the receiver's class) and method-level.
                                        let class_tp_names: FxHashSet<mir_types::Name> =
                                            crate::db::class_template_params(
                                                self.db,
                                                fqcn.as_ref(),
                                            )
                                            .map(|tps| {
                                                tps.iter()
                                                    .map(|tp| {
                                                        mir_types::Name::from(tp.name.as_ref())
                                                    })
                                                    .collect()
                                            })
                                            .unwrap_or_default();
                                        // Resolve the property's declared type against the
                                        // receiver's own concrete type args (e.g. `Box<int>`
                                        // binds `T -> int`) before deciding whether to skip:
                                        // a write through a receiver whose template args are
                                        // statically known should still be checked, not
                                        // unconditionally waved through just because the
                                        // docblock type mentions a template name.
                                        let class_tps = crate::db::effective_class_template_params(
                                            self.db,
                                            fqcn.as_ref(),
                                        )
                                        .map(|tps| tps.to_vec())
                                        .unwrap_or_default();
                                        let mut bindings = crate::generic::build_class_bindings(
                                            &class_tps,
                                            type_params,
                                        );
                                        for (k, v) in crate::db::inherited_template_bindings(
                                            self.db,
                                            fqcn.as_ref(),
                                            &bindings,
                                        ) {
                                            bindings.entry(k).or_insert(v);
                                        }
                                        let resolved_prop_ty = if bindings.is_empty() {
                                            prop_ty.clone()
                                        } else {
                                            prop_ty.substitute_templates(&bindings)
                                        };
                                        // Skip the check if the resolved prop_ty or ty still
                                        // references any unresolvable template param
                                        // (class-level or method-level). Inside a generic
                                        // class, $this carries no concrete type args, so class
                                        // templates in prop_ty can't be resolved there, and
                                        // method templates in ty are likewise unknown.
                                        let skip =
                                            type_refs_any_template(
                                                &resolved_prop_ty,
                                                &class_tp_names,
                                            ) || type_refs_any_template(&ty, &class_tp_names)
                                                || type_refs_any_template(
                                                    &ty,
                                                    &ctx.template_param_names,
                                                );
                                        // A docblock-only (`@var`) property
                                        // accepts null (implicit null default);
                                        // widen for the compatibility decision
                                        // only, keeping the declared type in the
                                        // emitted message.
                                        let compat_ty = if prop_has_native_type {
                                            resolved_prop_ty.clone()
                                        } else {
                                            let mut t = resolved_prop_ty.clone();
                                            t.add_type(Atomic::TNull);
                                            t
                                        };
                                        if !skip
                                            && !property_assign_compatible(&ty, &compat_ty, self.db)
                                        {
                                            if is_property_type_coercion(
                                                &ty,
                                                &resolved_prop_ty,
                                                self.db,
                                            ) {
                                                self.emit(
                                                    IssueKind::PropertyTypeCoercion {
                                                        property: prop_name.clone(),
                                                        expected: format!("{resolved_prop_ty}"),
                                                        actual: format!("{ty}"),
                                                    },
                                                    Severity::Info,
                                                    span,
                                                );
                                            } else {
                                                self.emit(
                                                    IssueKind::InvalidPropertyAssignment {
                                                        property: prop_name.clone(),
                                                        expected: format!("{resolved_prop_ty}"),
                                                        actual: format!("{ty}"),
                                                    },
                                                    Severity::Warning,
                                                    span,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Narrow the property type in prop_refined when the assignment is
                // compatible with the declared type (so the refined type is a valid
                // sub-type, e.g. assigning non-null to a nullable property).
                // Skip refinement on invalid assignments to avoid masking later errors.
                if let ExprKind::Variable(obj_var) = &pa.object.kind {
                    if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                        let obj_ty = ctx.get_var(obj_var.as_ref());
                        let declared_opt: Option<std::sync::Arc<mir_types::Type>> =
                            obj_ty.types.iter().find_map(|a| {
                                if let Atomic::TNamedObject { fqcn, .. } = a {
                                    let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                                    crate::db::find_property_in_chain(self.db, here, &prop_name)
                                        .and_then(|(_, p)| p.ty.clone())
                                } else {
                                    None
                                }
                            });
                        let should_refine = !ty.is_mixed()
                            && declared_opt
                                .as_deref()
                                .map(|declared| crate::subtype::is_subtype(self.db, &ty, declared))
                                .unwrap_or(true);
                        if should_refine {
                            ctx.set_prop_refined(obj_var.as_ref(), &prop_name, ty.clone());
                        } else {
                            // Assignment with incompatible or unknown (mixed) type: discard
                            // any stale guard-based narrowing so reads fall back to declared.
                            ctx.clear_prop_refined(obj_var.as_ref(), &prop_name);
                        }
                    }
                }
            }
            ExprKind::StaticPropertyAccess(spa) => {
                if let ExprKind::Identifier(id) = &spa.class.kind {
                    let resolved = crate::db::resolve_name(self.db, &self.file, id.as_ref());
                    let fqcn_opt: Option<std::sync::Arc<str>> = match resolved.as_str() {
                        "self" | "static" => {
                            ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone())
                        }
                        "parent" => ctx.parent_fqcn.clone(),
                        s => Some(std::sync::Arc::from(s)),
                    };
                    if let Some(fqcn) = fqcn_opt {
                        let prop_name_opt = match &spa.member.kind {
                            ExprKind::Variable(name) | ExprKind::Identifier(name) => {
                                Some(name.trim_start_matches('$').to_string())
                            }
                            _ => None,
                        };
                        if let Some(prop_name) = &prop_name_opt {
                            // Purity check: assigning to a static property in a @pure
                            // function. Unlike an instance property assignment (only
                            // impure through a parameter/captured receiver), a static
                            // property IS the shared external state — same as a
                            // global variable — so every write is impure, not just
                            // ones through a specific receiver.
                            if ctx.is_in_pure_fn {
                                self.emit(
                                    IssueKind::ImpureStaticPropertyAssignment {
                                        class: fqcn.to_string(),
                                        property: prop_name.clone(),
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                            // Without this, a static property write (Foo::$prop = ...,
                            // self::$prop = ..., static::$prop = ...) got no hover,
                            // go-to-definition, or find-all-references at all — unlike
                            // the read path (analyze_static_property_access), which
                            // records both. Key by the declaring owner, not the
                            // accessed-through class, matching the read path.
                            let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                            let prop_owner = crate::db::find_property_in_chain(self.db, here, prop_name)
                                .map(|(cls, _)| cls)
                                .unwrap_or_else(|| fqcn.clone());
                            self.record_ref(
                                std::sync::Arc::from(format!("prop:{}::{}", prop_owner, prop_name)),
                                spa.member.span,
                            );
                            self.record_symbol(
                                spa.member.span,
                                crate::symbol::ReferenceKind::PropertyAccess {
                                    class: prop_owner,
                                    property: std::sync::Arc::from(prop_name.as_str()),
                                },
                                ty.clone(),
                            );
                        }
                        if let Some(prop_name) = prop_name_opt.clone() {
                            let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                            if let Some((_, prop_def)) =
                                crate::db::find_property_in_chain(self.db, here, &prop_name)
                            {
                                let prop_has_native_type = prop_def.has_native_type;
                                if let Some(prop_ty) = prop_def.ty.as_deref() {
                                    if !prop_ty.is_mixed_not_template()
                                        && !ty.is_mixed_not_template()
                                    {
                                        let class_tp_names: FxHashSet<mir_types::Name> =
                                            crate::db::class_template_params(
                                                self.db,
                                                fqcn.as_ref(),
                                            )
                                            .map(|tps| {
                                                tps.iter()
                                                    .map(|tp| {
                                                        mir_types::Name::from(tp.name.as_ref())
                                                    })
                                                    .collect()
                                            })
                                            .unwrap_or_default();
                                        // A static access has no receiver instance to carry
                                        // type args, but an `@extends Box<int>` clause on the
                                        // accessed class itself still statically binds the
                                        // declaring class's template param — resolve that
                                        // before deciding whether to skip.
                                        let bindings = crate::db::inherited_template_bindings(
                                            self.db,
                                            fqcn.as_ref(),
                                            &FxHashMap::default(),
                                        );
                                        let resolved_prop_ty = if bindings.is_empty() {
                                            prop_ty.clone()
                                        } else {
                                            prop_ty.substitute_templates(&bindings)
                                        };
                                        let skip =
                                            type_refs_any_template(
                                                &resolved_prop_ty,
                                                &class_tp_names,
                                            ) || type_refs_any_template(&ty, &class_tp_names)
                                                || type_refs_any_template(
                                                    &ty,
                                                    &ctx.template_param_names,
                                                );
                                        // A docblock-only (`@var`) property
                                        // accepts null (implicit null default);
                                        // widen for the compatibility decision
                                        // only, keeping the declared type in the
                                        // emitted message.
                                        let compat_ty = if prop_has_native_type {
                                            resolved_prop_ty.clone()
                                        } else {
                                            let mut t = resolved_prop_ty.clone();
                                            t.add_type(Atomic::TNull);
                                            t
                                        };
                                        if !skip
                                            && !property_assign_compatible(&ty, &compat_ty, self.db)
                                        {
                                            if is_property_type_coercion(
                                                &ty,
                                                &resolved_prop_ty,
                                                self.db,
                                            ) {
                                                self.emit(
                                                    IssueKind::PropertyTypeCoercion {
                                                        property: prop_name.clone(),
                                                        expected: format!("{resolved_prop_ty}"),
                                                        actual: format!("{ty}"),
                                                    },
                                                    Severity::Info,
                                                    span,
                                                );
                                            } else {
                                                self.emit(
                                                    IssueKind::InvalidPropertyAssignment {
                                                        property: prop_name.clone(),
                                                        expected: format!("{resolved_prop_ty}"),
                                                        actual: format!("{ty}"),
                                                    },
                                                    Severity::Warning,
                                                    span,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Narrow the static property type the same way an instance
                        // property is narrowed on assignment (reusing prop_refined,
                        // keyed by the FQCN instead of a receiver variable name — a
                        // FQCN can never collide with a real PHP variable name).
                        if let Some(prop_name) = prop_name_opt {
                            let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
                            let declared_opt =
                                crate::db::find_property_in_chain(self.db, here, &prop_name)
                                    .and_then(|(_, p)| p.ty.clone());
                            let should_refine = !ty.is_mixed()
                                && declared_opt
                                    .as_deref()
                                    .map(|declared| {
                                        crate::subtype::is_subtype(self.db, &ty, declared)
                                    })
                                    .unwrap_or(true);
                            if should_refine {
                                ctx.set_prop_refined(fqcn.as_ref(), &prop_name, ty.clone());
                            } else {
                                ctx.clear_prop_refined(fqcn.as_ref(), &prop_name);
                            }
                        }
                    }
                }
            }
            ExprKind::ArrayAccess(aa) => {
                // Collect the full index chain from outermost to innermost.
                // For `$arr[$a][$b] = $val`, this gives [type($b), type($a)].
                // None means push notation (`[]`), which produces TList rather than TArray.
                // The base variable's key is the innermost (last in vec), and
                // intermediate indices are used to wrap the value type.
                let outer_key: Option<Type> = aa.index.as_ref().map(|idx| self.analyze(idx, ctx));
                let mut key_chain: Vec<Option<Type>> = vec![outer_key];
                // Parallel chain of literal array keys (same order as key_chain),
                // used to route a fully-literal nested write (`$arr['a']['b'] = $v`)
                // through a precise per-property update instead of widening the
                // whole outer shape.
                let mut literal_key_chain: Vec<Option<mir_types::ArrayKey>> = vec![aa
                    .index
                    .as_ref()
                    .and_then(|idx| super::helpers::literal_array_key_of_kind(&idx.kind))];
                let mut base: &Expr = &aa.array;
                loop {
                    match &base.kind {
                        ExprKind::Variable(name) => {
                            let name_str = name.trim_start_matches('$');
                            // Base key: innermost index in the chain (closest to $arr).
                            let base_key_opt = key_chain.last().unwrap().clone();
                            let base_key = base_key_opt.unwrap_or_else(Type::mixed);
                            // Only a single-level write ($arr[<key>] = $val, no
                            // nested chain) has a directly-known literal key —
                            // used to update just that one shape property
                            // in place instead of widening the whole shape.
                            let literal_key: Option<mir_types::ArrayKey> = if key_chain.len() == 1 {
                                aa.index.as_ref().and_then(|idx| match &idx.kind {
                                    ExprKind::String(s) => {
                                        Some(match super::helpers::canonical_int_array_key(s) {
                                            Some(i) => mir_types::ArrayKey::Int(i),
                                            None => mir_types::ArrayKey::String(
                                                std::sync::Arc::from(s.as_ref()),
                                            ),
                                        })
                                    }
                                    ExprKind::Int(i) => Some(mir_types::ArrayKey::Int(*i)),
                                    _ => None,
                                })
                            } else {
                                None
                            };
                            // Wrap the assigned value with intermediate keys, innermost
                            // (closest to the value) first. `key_chain` is populated
                            // outermost-AST-node-first, i.e. index 0 is the innermost path
                            // segment (`$a['x']['y']['z'] = 1` pushes 'z' before 'y' before
                            // 'x'), so iterating it in its natural order already applies
                            // keys innermost-to-outermost — do NOT reverse it, or a 3+-level
                            // chain wraps its middle keys in the wrong order.
                            // None entries ([] push) produce TList instead of TArray.
                            let mut wrapped_value = ty.clone();
                            for k_opt in key_chain[..key_chain.len() - 1].iter() {
                                wrapped_value = match k_opt {
                                    None => Type::single(Atomic::TList {
                                        value: Box::new(wrapped_value),
                                    }),
                                    Some(k) => Type::single(Atomic::TArray {
                                        key: Box::new(k.clone()),
                                        value: Box::new(wrapped_value),
                                    }),
                                };
                            }
                            if !ctx.var_is_defined(name_str) {
                                let name_sym = mir_types::Name::from(name_str);
                                let init_ty = match &key_chain.last().unwrap() {
                                    None => Type::single(Atomic::TList {
                                        value: Box::new(wrapped_value),
                                    }),
                                    Some(_) => Type::single(Atomic::TArray {
                                        key: Box::new(base_key),
                                        value: Box::new(wrapped_value),
                                    }),
                                };
                                std::sync::Arc::make_mut(&mut ctx.vars).insert(
                                    name_sym,
                                    mir_codebase::definitions::wrap_var_type(init_ty),
                                );
                                std::sync::Arc::make_mut(&mut ctx.assigned_vars).insert(name_sym);
                                let (line, col_start) = self.offset_to_line_col(base.span.start);
                                let (line_end, col_end) = self.offset_to_line_col(base.span.end);
                                ctx.record_var_location(
                                    name_str, line, col_start, line_end, col_end,
                                );
                            } else {
                                let current = ctx.get_var(name_str);
                                // Check if assigning to array offset of a non-array scalar
                                if !current.is_mixed()
                                    && !current.types.is_empty()
                                    && current.types.iter().all(|a| {
                                        matches!(
                                            a,
                                            Atomic::TInt
                                                | Atomic::TLiteralInt(_)
                                                | Atomic::TIntRange { .. }
                                                | Atomic::TPositiveInt
                                                | Atomic::TFloat
                                                | Atomic::TIntegralFloat
                                                | Atomic::TLiteralFloat(_, _)
                                                | Atomic::TBool
                                                | Atomic::TTrue
                                                | Atomic::TFalse
                                        )
                                    })
                                {
                                    self.emit(
                                        IssueKind::InvalidArrayAssignment {
                                            ty: current.to_string(),
                                        },
                                        Severity::Error,
                                        span,
                                    );
                                }
                                // A fully-literal nested write (`$arr['a']['b'] = $v`)
                                // can be routed through a precise per-property update
                                // at every level instead of widening the whole outer
                                // shape — try that first (innermost key first, i.e.
                                // the reverse of the outermost-first chain), falling
                                // back to the existing generic accumulator when the
                                // path isn't fully literal or doesn't cleanly resolve.
                                let nested_path: Option<Vec<mir_types::ArrayKey>> =
                                    if key_chain.len() > 1 {
                                        literal_key_chain
                                            .iter()
                                            .rev()
                                            .cloned()
                                            .collect::<Option<Vec<_>>>()
                                    } else {
                                        None
                                    };
                                let nested_update = nested_path.and_then(|path| {
                                    super::helpers::set_nested_keyed_value(&current, &path, &ty)
                                });
                                let updated = match nested_update {
                                    Some(updated) => updated,
                                    None => match &key_chain.last().unwrap() {
                                        None => widen_array_as_list(&current, &wrapped_value),
                                        Some(_) => widen_array_with_value_and_key(
                                            &current,
                                            &wrapped_value,
                                            &base_key,
                                            literal_key.as_ref(),
                                        ),
                                    },
                                };
                                ctx.set_var(name_str, updated);
                            }
                            break;
                        }
                        ExprKind::ArrayAccess(inner) => {
                            let inner_key: Option<Type> =
                                inner.index.as_ref().map(|idx| self.analyze(idx, ctx));
                            literal_key_chain.push(inner.index.as_ref().and_then(|idx| {
                                super::helpers::literal_array_key_of_kind(&idx.kind)
                            }));
                            key_chain.push(inner_key);
                            base = &inner.array;
                        }
                        _ => break,
                    }
                }
            }
            ExprKind::VariableVariable(inner) => {
                // A variable-variable assignment may define arbitrarily-named
                // variables (e.g. `${$key} = …` or `${"$key"} = …`). Once seen,
                // later reads of otherwise-unknown variables must not be reported
                // as undefined — we cannot prove they were not defined here.
                ctx.has_dynamic_var_def = true;
                if let Some(var_name) = extract_simple_var(inner) {
                    ctx.read_vars
                        .insert(mir_types::Name::from(var_name.as_str()));
                    ctx.mark_consumed(&var_name);
                    let var_ty = ctx.get_var(&var_name);
                    for atomic in &var_ty.types {
                        if let Atomic::TLiteralString(accessed_var_name) = atomic {
                            ctx.set_var(accessed_var_name.as_ref(), ty.clone());
                            let (line, col_start) = self.offset_to_line_col(target.span.start);
                            let (line_end, col_end) = self.offset_to_line_col(target.span.end);
                            ctx.record_var_location(
                                accessed_var_name,
                                line,
                                col_start,
                                line_end,
                                col_end,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
