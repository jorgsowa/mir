use super::helpers::{
    as_concat_str, definite_key_state, extract_simple_var, extract_string_from_expr,
    infer_arithmetic, infer_div, infer_int_range_arithmetic, is_non_empty_when_concat,
    is_property_type_coercion, literal_array_key_of_kind, property_assign_compatible,
    type_refs_any_template, widen_array_as_list, widen_array_with_value_and_key, DefiniteKeyState,
};
use super::ExpressionAnalyzer;
use crate::db::MirDatabase;
use crate::flow_state::FlowState;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};
use php_ast::ast::{AssignOp, BinaryOp};
use php_ast::owned::{AssignExpr, Expr, ExprKind};
use php_ast::Span;
use rustc_hash::{FxHashMap, FxHashSet};

/// Taint every plain-variable leaf of a (possibly nested) array-destructuring
/// target — `[$a, $b] = $tainted;`, `['x' => $a, 'y' => [$b, $c]] = $tainted;`
/// — a element that's itself a nested `Array` recurses; a `PropertyAccess`/
/// `ArrayAccess` element (`[$obj->prop] = $tainted;`) is conservatively
/// skipped, matching how the plain-assignment case only taints a bare
/// variable or property, never an arbitrary nested write target.
pub(crate) fn taint_destructured_targets(target: &Expr, ctx: &mut FlowState) {
    let ExprKind::Array(elements) = &target.kind else {
        return;
    };
    for elem in elements.iter() {
        match &elem.value.kind {
            ExprKind::Variable(name) => ctx.taint_var(name.as_ref()),
            ExprKind::Array(_) => taint_destructured_targets(&elem.value, ctx),
            _ => {}
        }
    }
}

/// Resolve a `self::$prop`/`static::$prop`/`parent::$prop`/`Foo::$prop`
/// static-property target to its owning FQCN + bare property name — shared
/// resolution logic for the taint helpers below, mirroring the inline
/// version already duplicated in the `Assign`/`Concat` arms' own
/// `StaticPropertyAccess` match arms.
fn resolve_static_prop_target(
    spa: &php_ast::owned::StaticAccessExpr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(std::sync::Arc<str>, String)> {
    let ExprKind::Identifier(id) = &spa.class.kind else {
        return None;
    };
    let resolved = crate::db::resolve_name(db, file, id.as_ref());
    let fqcn = match resolved.as_str() {
        "self" | "static" => ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone()),
        "parent" => ctx.parent_fqcn.clone(),
        s => Some(std::sync::Arc::from(s)),
    }?;
    let prop_name = match &spa.member.kind {
        ExprKind::Variable(name) | ExprKind::Identifier(name) => {
            Some(name.trim_start_matches('$').to_string())
        }
        _ => None,
    }?;
    Some((fqcn, prop_name))
}

/// Whether a compound-assignment target's CURRENT value (before the
/// operation) is already tainted — used to implement "sticky" taint
/// (tainted afterwards if either the old value or the new RHS was), the
/// same semantics `.=` already applies to a bare variable, but generalized
/// to every trackable target shape for the arithmetic/`??=` arms below.
fn target_is_currently_tainted(
    target: &Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
    file: &str,
) -> bool {
    match &target.kind {
        ExprKind::Variable(name) => ctx.is_tainted(name.trim_start_matches('$')),
        ExprKind::PropertyAccess(pa) => {
            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                extract_string_from_expr(&pa.property)
                    .is_some_and(|prop| ctx.is_prop_tainted(obj_var.trim_start_matches('$'), &prop))
            } else {
                false
            }
        }
        ExprKind::StaticPropertyAccess(spa) => resolve_static_prop_target(spa, ctx, db, file)
            .is_some_and(|(fqcn, prop)| ctx.is_static_prop_tainted(&fqcn, &prop)),
        _ => false,
    }
}

/// `$arr['k'] = $tainted;` / `$arr['k'] .= $tainted;` / `$arr['k'] += $tainted;`
/// taints the whole container at the same coarse, whole-container
/// granularity the Array-literal taint check already uses for any tainted
/// element — but every one of the three array-element-write taint arms
/// below only ever matched a plain-variable base, silently dropping taint
/// for a property or static-property base (`$this->items['id'] =
/// $tainted;`, `self::$items['id'] = $tainted;`), even though the READ
/// side already supports both.
fn taint_array_write_base(base: &Expr, ctx: &mut FlowState, db: &dyn MirDatabase, file: &str) {
    match &base.kind {
        ExprKind::Variable(name) => ctx.taint_var(name.trim_start_matches('$')),
        ExprKind::PropertyAccess(pa) => {
            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                    ctx.taint_prop(obj_var.trim_start_matches('$'), &prop_name);
                }
            }
        }
        ExprKind::StaticPropertyAccess(spa) => {
            if let Some((fqcn, prop_name)) = resolve_static_prop_target(spa, ctx, db, file) {
                ctx.taint_static_prop(&fqcn, &prop_name);
            }
        }
        _ => {}
    }
}

/// Apply a compound-assignment's taint outcome to its target — shared by
/// the arithmetic (`+=` family) and `??=` arms, both of which need the same
/// four-way target-shape taint set/clear that `.=`'s own arm already
/// inlines for itself (kept separate there since it sits alongside
/// concat-specific string-length logic).
fn apply_compound_assign_taint(
    target: &Expr,
    should_taint: bool,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    match &target.kind {
        ExprKind::Variable(name) => {
            if should_taint {
                ctx.taint_var(name.as_ref());
            } else {
                ctx.clear_var_taint(name.as_ref());
            }
        }
        ExprKind::PropertyAccess(pa) => {
            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                    let obj_var = obj_var.trim_start_matches('$');
                    if should_taint {
                        ctx.taint_prop(obj_var, &prop_name);
                    } else {
                        ctx.clear_prop_taint(obj_var, &prop_name);
                    }
                }
            }
        }
        // Coarse, monotonic array-element taint (matching every other
        // taint-propagating array-element arm) — only ever set, never
        // cleared, since a single tainted element must not clean the whole
        // container just because THIS write happened to be untainted.
        ExprKind::ArrayAccess(aa) if should_taint => {
            taint_array_write_base(&aa.array, ctx, db, file);
        }
        ExprKind::StaticPropertyAccess(spa) => {
            if let Some((fqcn, prop_name)) = resolve_static_prop_target(spa, ctx, db, file) {
                if should_taint {
                    ctx.taint_static_prop(&fqcn, &prop_name);
                } else {
                    ctx.clear_static_prop_taint(&fqcn, &prop_name);
                }
            }
        }
        _ => {}
    }
}

/// Walk through a chain of property accesses (`$this->cache->v`'s object is
/// `$this->cache`, whose own object is `$this`) to find the root variable
/// name, or `None` if the chain doesn't bottom out in a bare variable (e.g.
/// a method-call result). Lets a purity/immutability check that only cares
/// about "is this ultimately reachable from `$this`/a parameter" match a
/// chained receiver the same way it already matches a direct one.
pub(crate) fn root_receiver_var(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(name.as_ref()),
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            root_receiver_var(&pa.object)
        }
        // `$this->caches[0]->v = 5` — an array-index hop in the middle of the
        // chain (`$this->caches[0]`) is still reachable from `$this`/a
        // parameter, same as a bare property hop; walk through it the same
        // way instead of bailing out to `None`.
        ExprKind::ArrayAccess(aa) => root_receiver_var(&aa.array),
        _ => None,
    }
}

/// Resolve a (possibly chained) property-access receiver's declared type —
/// e.g. the `$this->cache` part of `$this->cache->v` — by walking each hop's
/// declared property type via `find_property_in_chain`, instead of requiring
/// the receiver to literally BE a bare variable. Doesn't consult
/// `ctx.get_prop_refined` (unlike `narrowing::resolve_prop_current_type`):
/// callers (a readonly check, and `taint.rs`'s taint-source method-call
/// check) both only care about the property's *declared* type, not a
/// condition-narrowed one. Not re-running `self.analyze` here is
/// deliberate — the operand was already analyzed by the surrounding read,
/// and a fresh `analyze` call would double-report its diagnostics.
pub(crate) fn resolve_chained_receiver_type(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn MirDatabase,
) -> Option<Type> {
    match &expr.kind {
        ExprKind::Variable(name) => Some(ctx.get_var(name)),
        ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
            let obj_ty = resolve_chained_receiver_type(&pa.object, ctx, db)?;
            let prop_name = extract_string_from_expr(&pa.property)?;
            let mut result = Type::empty();
            for atomic in &obj_ty.types {
                if let Atomic::TNamedObject { fqcn, .. }
                | Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } = atomic
                {
                    if let Some((_, prop_def)) = crate::db::find_property_in_chain(
                        db,
                        crate::db::Fqcn::from_str(db, fqcn.as_ref()),
                        &prop_name,
                    ) {
                        if let Some(ty) = prop_def.ty.as_deref() {
                            result.merge_with(ty);
                        }
                    }
                }
            }
            Some(result)
        }
        // `$this->repos['main']->getInput()` — an array-index hop in the
        // middle of the chain is still reachable from `$this`/a parameter,
        // same as `root_receiver_var`'s own array-index arm above; this
        // resolver had no counterpart, so a chain with an index hop
        // silently broke off with `None` instead of yielding the element
        // type callers (e.g. the `@taint-source` method-call check) need.
        ExprKind::ArrayAccess(aa) => {
            let base_ty = resolve_chained_receiver_type(&aa.array, ctx, db)?;
            let mut result = Type::empty();
            for atomic in &base_ty.types {
                match atomic {
                    Atomic::TArray { value, .. } | Atomic::TNonEmptyArray { value, .. } => {
                        result.merge_with(value);
                    }
                    Atomic::TList { value } | Atomic::TNonEmptyList { value } => {
                        result.merge_with(value);
                    }
                    Atomic::TKeyedArray { properties, .. } => {
                        for prop in properties.values() {
                            result.merge_with(&prop.ty);
                        }
                    }
                    _ => {}
                }
            }
            Some(result)
        }
        // `$http->params()->get('id')` — an intermediate METHOD-CALL hop in
        // the chain (as opposed to a property/array-index hop, both already
        // handled above) had no counterpart either, so a `@taint-source`
        // method resolved through one more hop than a bare property/array
        // chain silently broke off with `None`. Resolves the called
        // method's declared return type, same shape as the property arm's
        // `find_property_in_chain` lookup.
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            let obj_ty = resolve_chained_receiver_type(&mc.object, ctx, db)?;
            let ExprKind::Identifier(method_name) = &mc.method.kind else {
                return None;
            };
            let method_lower = crate::util::php_ident_lowercase(method_name.as_ref());
            let mut result = Type::empty();
            for atomic in &obj_ty.types {
                if let Atomic::TNamedObject { fqcn, .. }
                | Atomic::TSelf { fqcn }
                | Atomic::TStaticObject { fqcn }
                | Atomic::TParent { fqcn } = atomic
                {
                    if let Some((_, method_def)) = crate::db::find_method_respecting_precedence(
                        db,
                        crate::db::Fqcn::from_str(db, fqcn.as_ref()),
                        &method_lower,
                    ) {
                        if let Some(ty) = method_def.return_type.as_deref() {
                            result.merge_with(ty);
                        }
                    }
                }
            }
            Some(result)
        }
        _ => None,
    }
}

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_assign(
        &mut self,
        a: &AssignExpr,
        expr_span: Span,
        ctx: &mut FlowState,
    ) -> Type {
        let rhs_tainted = crate::taint::is_expr_tainted(&a.value, ctx, self.db, &self.file);
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
                // `$x =& $this->prop;` — record the alias so a later PLAIN
                // `$x = value` write (which mutates `$this->prop` through the
                // reference, not just `$x` itself) can run the same
                // purity/immutability gate a direct `$this->prop = value`
                // write already does. Narrow: only this one AST-visible
                // shape (a bare local variable ref-aliased directly to a
                // var-receiver property) is tracked.
                if a.by_ref {
                    if let ExprKind::Variable(target_name) = &a.target.kind {
                        if let ExprKind::PropertyAccess(pa) = &a.value.kind {
                            if let ExprKind::Variable(recv_name) = &pa.object.kind {
                                if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                                    ctx.set_ref_alias(target_name, recv_name, &prop_name);
                                }
                            }
                        }
                    }
                }
                // `$clone = clone $this;` — record that `$clone` directly
                // holds a fresh, unaliased clone, so a later write through it
                // (the standard immutable "wither" idiom) isn't mistaken for
                // an externally-visible mutation. Any other plain reassignment
                // of the same variable clears the marker: the new value may
                // not be a fresh clone.
                if !a.by_ref {
                    if let ExprKind::Variable(target_name) = &a.target.kind {
                        if matches!(&a.value.kind, ExprKind::Clone(_) | ExprKind::CloneWith(..)) {
                            ctx.mark_cloned_local(target_name);
                        } else {
                            ctx.clear_cloned_local(target_name);
                        }
                    }
                }
                self.assign_to_target(&a.target, rhs_ty.clone(), ctx, expr_span);
                // A PLAIN (non-`=&`) write to a variable already ref-aliased
                // to a property mutates that property through the reference
                // — run the same purity/immutability gate a direct
                // `$this->prop = value` write already does. The `=&`
                // statement itself (handled above) only creates the alias;
                // it doesn't write the property's value, so it's excluded.
                if !a.by_ref {
                    if let ExprKind::Variable(name) = &a.target.kind {
                        if let Some((recv, prop)) = ctx.get_ref_alias(name) {
                            self.check_property_write_purity_by_name(
                                recv.as_ref(),
                                prop.as_ref(),
                                ctx,
                                expr_span,
                            );
                        }
                    }
                }
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
                        } else {
                            // Overwritten with a proven-clean value —
                            // don't let stale taint survive.
                            ctx.clear_var_taint(name.as_ref());
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
                    // List/array destructuring (`[$a, $b] = $arr;`,
                    // `['x' => $a] = $arr;`) from a tainted source taints
                    // every destructured variable — this was the one target
                    // shape with no taint propagation at all, unlike plain
                    // variable/property assignment just above.
                    ExprKind::Array(_) if rhs_tainted => {
                        taint_destructured_targets(&a.target, ctx);
                    }
                    // `$arr['k'] = $tainted;` — taint the whole array (same
                    // coarse, whole-container granularity the Array-literal
                    // taint check already uses for any tainted element), so
                    // a later read of ANY key sees it as tainted. Only a
                    // simple-variable base is tracked, matching every other
                    // taint-propagating target arm above.
                    ExprKind::ArrayAccess(aa) if rhs_tainted => {
                        taint_array_write_base(&aa.array, ctx, self.db, &self.file);
                    }
                    // `self::$prop = $tainted;` / `Foo::$prop = $tainted;` —
                    // static properties were entirely untracked for taint,
                    // unlike instance properties (the arm above).
                    ExprKind::StaticPropertyAccess(spa) => {
                        if let ExprKind::Identifier(id) = &spa.class.kind {
                            let resolved =
                                crate::db::resolve_name(self.db, &self.file, id.as_ref());
                            let fqcn_opt: Option<std::sync::Arc<str>> = match resolved.as_str() {
                                "self" | "static" => {
                                    ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone())
                                }
                                "parent" => ctx.parent_fqcn.clone(),
                                s => Some(std::sync::Arc::from(s)),
                            };
                            if let Some(fqcn) = fqcn_opt {
                                if let Some(prop_name) = match &spa.member.kind {
                                    ExprKind::Variable(name) | ExprKind::Identifier(name) => {
                                        Some(name.trim_start_matches('$').to_string())
                                    }
                                    _ => None,
                                } {
                                    if rhs_tainted {
                                        ctx.taint_static_prop(&fqcn, &prop_name);
                                    } else {
                                        ctx.clear_static_prop_taint(&fqcn, &prop_name);
                                    }
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
                    // `.=` on a by-ref PARAMETER or superglobal mutates it
                    // exactly as much as a plain `=` overwrite does, but this
                    // fast path (unlike the non-variable branch below, which
                    // falls through to `assign_to_target`) never routed
                    // through any purity check at all.
                    self.check_var_write_purity(&var_name, ctx, expr_span);
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
                    // `.=`'s result keeps the OLD value's content (unlike plain
                    // `=`, which fully replaces it) — so it stays tainted if
                    // either side was, not just the RHS.
                    if rhs_tainted || ctx.is_tainted(&var_name) {
                        ctx.taint_var(&var_name);
                    } else {
                        ctx.clear_var_taint(&var_name);
                    }
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
                    // Same "sticky" taint reasoning as the simple-variable branch
                    // above, mirrored per target shape the same way plain `=`
                    // already is (property/static-property tracked precisely and
                    // clearable; array-element taint is coarse and monotonic, so
                    // it only ever needs setting, never clearing).
                    match &a.target.kind {
                        ExprKind::PropertyAccess(pa) => {
                            if let ExprKind::Variable(obj_var) = &pa.object.kind {
                                if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                                    let obj_var = obj_var.trim_start_matches('$');
                                    if rhs_tainted || ctx.is_prop_tainted(obj_var, &prop_name) {
                                        ctx.taint_prop(obj_var, &prop_name);
                                    } else {
                                        ctx.clear_prop_taint(obj_var, &prop_name);
                                    }
                                }
                            }
                        }
                        ExprKind::ArrayAccess(aa) if rhs_tainted => {
                            taint_array_write_base(&aa.array, ctx, self.db, &self.file);
                        }
                        ExprKind::StaticPropertyAccess(spa) => {
                            if let ExprKind::Identifier(id) = &spa.class.kind {
                                let resolved =
                                    crate::db::resolve_name(self.db, &self.file, id.as_ref());
                                let fqcn_opt: Option<std::sync::Arc<str>> = match resolved.as_str()
                                {
                                    "self" | "static" => {
                                        ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone())
                                    }
                                    "parent" => ctx.parent_fqcn.clone(),
                                    s => Some(std::sync::Arc::from(s)),
                                };
                                if let Some(fqcn) = fqcn_opt {
                                    if let Some(prop_name) = match &spa.member.kind {
                                        ExprKind::Variable(name) | ExprKind::Identifier(name) => {
                                            Some(name.trim_start_matches('$').to_string())
                                        }
                                        _ => None,
                                    } {
                                        if rhs_tainted
                                            || ctx.is_static_prop_tainted(&fqcn, &prop_name)
                                        {
                                            ctx.taint_static_prop(&fqcn, &prop_name);
                                        } else {
                                            ctx.clear_static_prop_taint(&fqcn, &prop_name);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
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
                // `$a += $tainted` reads the OLD value before writing, same
                // "sticky" reasoning `.=` already applies: the result stays
                // tainted if either side was, unlike plain `=` which fully
                // replaces the value.
                let should_taint =
                    rhs_tainted || target_is_currently_tainted(&a.target, ctx, self.db, &self.file);
                self.assign_to_target(&a.target, result_ty.clone(), ctx, expr_span);
                apply_compound_assign_taint(&a.target, should_taint, ctx, self.db, &self.file);
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
                // `$arr['a'] ??= 'y'` on a single-level literal array offset: if the
                // array's shape proves the key is definitely absent (or definitely
                // present with a non-null value), we know for certain whether the
                // right-hand side runs — no need to fall back to a plain union of
                // "maybe the old value, maybe the new one".
                let literal_offset_state = match &a.target.kind {
                    ExprKind::ArrayAccess(aa) => match (&aa.array.kind, aa.index.as_deref()) {
                        (ExprKind::Variable(name), Some(idx)) => {
                            literal_array_key_of_kind(&idx.kind).and_then(|key| {
                                let base = ctx.get_var(name.trim_start_matches('$'));
                                definite_key_state(&base, &key)
                            })
                        }
                        _ => None,
                    },
                    _ => None,
                };
                let lhs_ty = self.with_existence_check(|ea| ea.analyze(&a.target, ctx));
                // Taint mirrors the same three-way split as the type merge
                // below: a definite-absent/undefined target takes exactly
                // the RHS, a definite-present one keeps its OLD value
                // untouched (no taint change either), and an uncertain one
                // could end up as either — sticky, same as the arithmetic
                // arm. Computed before `merged` so matching against
                // `literal_offset_state` here doesn't fight its later
                // by-value match (`Type` isn't `Copy`).
                let should_taint = if is_undefined_var
                    || matches!(literal_offset_state, Some(DefiniteKeyState::Absent))
                {
                    Some(rhs_tainted)
                } else if matches!(literal_offset_state, Some(DefiniteKeyState::Present(_))) {
                    None
                } else {
                    Some(
                        rhs_tainted
                            || target_is_currently_tainted(&a.target, ctx, self.db, &self.file),
                    )
                };
                let merged = if is_undefined_var
                    || matches!(literal_offset_state, Some(DefiniteKeyState::Absent))
                {
                    rhs_ty.clone()
                } else if let Some(DefiniteKeyState::Present(ty)) = literal_offset_state {
                    ty
                } else {
                    Type::merge(&lhs_ty.remove_null(), &rhs_ty)
                };
                // Route through assign_to_target (not just the simple-variable case) so
                // property/array targets are also narrowed — e.g. `$this->x ??= 'y'`
                // should leave $this->x non-null afterwards, not just plain `$x ??= 'y'`.
                self.assign_to_target(&a.target, merged.clone(), ctx, expr_span);
                if let Some(should_taint) = should_taint {
                    apply_compound_assign_taint(&a.target, should_taint, ctx, self.db, &self.file);
                }
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

    /// Purity/immutability checks for writing to a property, shared between a
    /// plain `$obj->prop = x` assignment and a mutation reached through a
    /// non-assignment write path on the same property (array-index write,
    /// `unset()`) that resolves to the same receiver+property but doesn't go
    /// through `assign_to_target`'s own `PropertyAccess` arm.
    pub(crate) fn check_property_write_purity(
        &mut self,
        pa: &php_ast::owned::PropertyAccessExpr,
        ctx: &FlowState,
        span: Span,
    ) {
        // `$this->cache->v = 5` (a chained, non-`$this`-literal receiver)
        // still mutates state reachable from `$this`/a parameter, same as a
        // direct `$this->prop = x` — walk through any nested property-access
        // chain to find the root variable, instead of only matching when
        // `pa.object` IS that bare variable.
        if let Some(recv_name) = root_receiver_var(&pa.object) {
            if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                self.check_property_write_purity_by_name(recv_name, &prop_name, ctx, span);
            }
        }
        // Cross-class immutable write: a write to a non-`$this` receiver of
        // an immutable-tagged class is forbidden from ANY caller, not just
        // when reached through a plain `$obj->prop = x` assignment — an
        // array-index write, `++`/`--`, a by-ref call argument, or a by-ref
        // `foreach` over the same property all mutate it identically, but
        // only the plain-assignment arm ever ran this check.
        let is_this_receiver = matches!(
            &pa.object.kind,
            ExprKind::Variable(n) if n.trim_start_matches('$') == "this"
        );
        // A write through a variable directly holding a fresh `clone $this`
        // (the standard immutable "wither" idiom: clone, mutate the clone,
        // return it) isn't an externally-visible mutation — the clone is a
        // new, unaliased object nothing else can observe yet, unlike a write
        // reached through a parameter or another already-shared reference.
        let is_cloned_local_receiver = matches!(
            &pa.object.kind,
            ExprKind::Variable(n) if ctx.is_cloned_local(n)
        );
        if !is_this_receiver && !is_cloned_local_receiver {
            if let Some(prop_name) = extract_string_from_expr(&pa.property) {
                if let Some(obj_ty) = resolve_chained_receiver_type(&pa.object, ctx, self.db) {
                    for atomic in &obj_ty.types {
                        if let Atomic::TNamedObject { fqcn, .. }
                        | Atomic::TSelf { fqcn }
                        | Atomic::TStaticObject { fqcn }
                        | Atomic::TParent { fqcn } = atomic
                        {
                            if crate::db::class_is_immutable(self.db, fqcn.as_ref()) {
                                let receiver =
                                    crate::parser::span_text(self.source, pa.object.span)
                                        .unwrap_or_else(|| "the receiver".to_string());
                                self.emit(
                                    IssueKind::ImmutablePropertyModification {
                                        receiver,
                                        property: prop_name.clone(),
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

    /// Like `check_property_write_purity`, but takes the receiver/property as
    /// plain strings instead of requiring a real `PropertyAccessExpr` AST
    /// node — lets a write reached through a local variable ref-aliased to
    /// a property (`$x =& $this->prop; $x = 5;`) reuse the same gate without
    /// synthesizing a fake node.
    /// `++`/`--`, `unset()`, and an array-index write through a property
    /// base (`$this->items[] = x`) never go through `assign_to_target`'s own
    /// `PropertyAccess` arm, which is the ONLY place a plain `=`/compound-op
    /// write's readonly violation is caught — so all three silently bypassed
    /// `@readonly` enforcement entirely. Walks a chained receiver
    /// (`$this->cache->v`) via `resolve_chained_receiver_type`, mirroring
    /// `check_property_write_purity`'s own chain-walk — a fresh
    /// `self.analyze(&pa.object, ...)` here would re-run (and double-report)
    /// whatever reference/diagnostic recording already happened when the
    /// surrounding read of the operand ran.
    /// None of these three write shapes can legally be a property's first
    /// (initializing) write — they all read the current value first (an
    /// implicit read for `++`/array-append, an explicit one for a keyed
    /// array write/unset) — so unlike the plain-assignment case, there's no
    /// "allowed in the declaring scope" exception to thread through here.
    pub(crate) fn check_property_readonly_write(
        &mut self,
        pa: &php_ast::owned::PropertyAccessExpr,
        ctx: &FlowState,
        span: Span,
    ) {
        let Some(prop_name) = extract_string_from_expr(&pa.property) else {
            return;
        };
        let Some(obj_ty) = resolve_chained_receiver_type(&pa.object, ctx, self.db) else {
            return;
        };
        for atomic in &obj_ty.types {
            if let Atomic::TNamedObject { fqcn, .. }
            | Atomic::TSelf { fqcn }
            | Atomic::TStaticObject { fqcn }
            | Atomic::TParent { fqcn } = atomic
            {
                let db = self.db;
                if let Some((owner, prop_def)) = crate::db::find_property_in_chain(
                    db,
                    crate::db::Fqcn::from_str(db, fqcn.as_ref()),
                    &prop_name,
                ) {
                    if prop_def.is_readonly {
                        self.emit(
                            IssueKind::ReadonlyPropertyAssignment {
                                class: owner.to_string(),
                                property: prop_name.clone(),
                            },
                            Severity::Error,
                            span,
                        );
                    }
                }
            }
        }
    }

    /// A by-ref call argument that's a property (`array_push($this->items,
    /// $n)`, `sort($this->items)`) mutates that property exactly as much as
    /// an explicit `$this->items = …` write would — but every by-ref
    /// write-back site only ever matched `ExprKind::Variable` for the output
    /// type, silently skipping (never even reading) a property argument, so
    /// passing one by reference bypassed purity/immutability/readonly
    /// checking entirely. A no-op for any other argument shape (the common
    /// `ExprKind::Variable` case is already handled by each write-back site
    /// itself).
    pub(crate) fn check_byref_arg_purity(&mut self, arg_expr: &Expr, ctx: &FlowState, span: Span) {
        // `sort($this->cache['x']['y'])` — any depth of array-index-into-
        // property nesting mutates that property's contents exactly as much
        // as a direct property argument (`sort($this->cache)`) does. Unwrap
        // every `ArrayAccess` level (not just one) down to the real base
        // before checking it, mirroring `assign_to_target`'s own base-walk
        // loop for a plain nested-index write.
        let mut base = arg_expr;
        while let ExprKind::ArrayAccess(aa) = &base.kind {
            base = &aa.array;
        }
        match &base.kind {
            ExprKind::PropertyAccess(pa) => {
                self.check_property_write_purity(pa, ctx, span);
                self.check_property_readonly_write(pa, ctx, span);
            }
            // `array_push(self::$queue, $x)` — a static-property by-ref
            // argument mutates that property exactly as much as
            // `self::$queue = …` would.
            ExprKind::StaticPropertyAccess(spa) => {
                self.check_static_prop_byref_purity(spa, ctx, span);
            }
            // A bare variable base covers every mutation shape that reuses
            // this function for a plain by-ref PARAMETER or superglobal
            // (`++`/`--` and `foreach (&$v)`, both routed here by their own
            // call sites, plus a direct by-ref call argument like
            // `sort($items)`) — previously unchecked entirely, unlike the
            // property/static-property arms above.
            ExprKind::Variable(name) => {
                self.check_var_write_purity(name.trim_start_matches('$'), ctx, span);
            }
            _ => {}
        }
    }

    /// Purity/mutation-free check for a bare variable mutated OUTSIDE the
    /// plain `$x = value` assignment shape (`assign_to_target`'s own
    /// `Variable` arm already covers that one) — a `.=` fast path, a
    /// by-ref call argument/`++`/`--`/`foreach(&$v)` (via
    /// `check_byref_arg_purity`), or an `unset()` all mutate a by-ref
    /// PARAMETER or a superglobal exactly as much as a plain overwrite does.
    pub(crate) fn check_var_write_purity(&mut self, name: &str, ctx: &FlowState, span: Span) {
        if !(ctx.is_in_pure_fn || ctx.is_in_immutable_method) {
            return;
        }
        let name_sym = mir_types::Name::from(name);
        if ctx.byref_param_names.contains(&name_sym) && ctx.param_names.contains(&name_sym) {
            self.emit(
                IssueKind::ImpureByRefAssignment {
                    variable: name.to_string(),
                },
                Severity::Warning,
                span,
            );
        } else if crate::util::is_superglobal_name(name) {
            self.emit(
                IssueKind::ImpureGlobalVariable {
                    variable: name.to_string(),
                },
                Severity::Warning,
                span,
            );
        }
    }

    /// Purity/readonly checks for a static property mutated via a by-ref
    /// call argument (`array_push(self::$queue, $x)`, `sort(Bag::$items)`)
    /// — mirrors the plain `self::$prop = …` write arm's own inline checks
    /// (`ImpureStaticPropertyAssignment` + readonly lookup), which this
    /// by-ref path never reached at all.
    fn check_static_prop_byref_purity(
        &mut self,
        spa: &php_ast::owned::StaticAccessExpr,
        ctx: &FlowState,
        span: Span,
    ) {
        let Some((fqcn, prop_name)) = resolve_static_prop_target(spa, ctx, self.db, &self.file)
        else {
            return;
        };
        // A static property is shared external state exactly like a global
        // variable — @mutation-free ("nothing external") forbids writing it
        // just as much as @pure does, not just a $this-property write.
        if ctx.is_in_pure_fn || ctx.is_in_immutable_method {
            self.emit(
                IssueKind::ImpureStaticPropertyAssignment {
                    class: fqcn.to_string(),
                    property: prop_name.clone(),
                },
                Severity::Warning,
                span,
            );
        }
        let here = crate::db::Fqcn::from_str(self.db, fqcn.as_ref());
        if let Some((owner, prop_def)) =
            crate::db::find_property_in_chain(self.db, here, &prop_name)
        {
            if prop_def.is_readonly {
                self.emit(
                    IssueKind::ReadonlyPropertyAssignment {
                        class: owner.to_string(),
                        property: prop_name,
                    },
                    Severity::Error,
                    span,
                );
            }
        }
    }

    pub(crate) fn check_property_write_purity_by_name(
        &mut self,
        recv_name: &str,
        prop_name: &str,
        ctx: &FlowState,
        span: Span,
    ) {
        let recv_stripped = recv_name.trim_start_matches('$');
        // Purity check: assigning to a parameter's property in a @pure function.
        if ctx.is_in_pure_fn
            && ctx
                .param_names
                .contains(&mir_types::Name::from(recv_stripped))
        {
            self.emit(
                IssueKind::ImpurePropertyAssignment {
                    property: prop_name.to_string(),
                },
                Severity::Warning,
                span,
            );
        }
        // External-mutation-free check: assigning to a parameter's property in
        // a @psalm-external-mutation-free method is forbidden.
        if ctx.is_in_external_mutation_free_method
            && recv_stripped != "this"
            && ctx
                .param_names
                .contains(&mir_types::Name::from(recv_stripped))
        {
            self.emit(
                IssueKind::ImpurePropertyAssignment {
                    property: prop_name.to_string(),
                },
                Severity::Warning,
                span,
            );
        }
        // Immutability check: assigning to $this->prop in a @psalm-immutable class.
        if ctx.is_in_immutable_method && recv_stripped == "this" {
            self.emit(
                IssueKind::ImmutablePropertyModification {
                    receiver: "$this".to_string(),
                    property: prop_name.to_string(),
                },
                Severity::Warning,
                span,
            );
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
                // Purity check: a bare (whole-array) superglobal write
                // (`$_SESSION = [];`) reaches the same external mutable
                // state as `$_SESSION['x'] = ...`; the indexed-write shape
                // is already checked in this same match a few arms up
                // (ArrayAccess), this is its whole-array-overwrite sibling.
                if (ctx.is_in_pure_fn || ctx.is_in_immutable_method)
                    && crate::util::is_superglobal_name(&name_str)
                {
                    self.emit(
                        IssueKind::ImpureGlobalVariable {
                            variable: name_str.clone(),
                        },
                        Severity::Warning,
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
                // Without this, hover/go-to-definition on the variable name worked on
                // the read side (analyze_variable) but not on a plain-assignment write
                // target ($x = ... / list()/array-destructuring targets), unlike the
                // already-fixed property write case just below.
                self.record_symbol(
                    target.span,
                    crate::symbol::ReferenceKind::Variable(std::sync::Arc::from(name_str.as_str())),
                    ty.clone(),
                );
                ctx.set_var(&name_str, ty);
                let (line, col_start) = self.offset_to_line_col(target.span.start);
                let (line_end, col_end) = self.offset_to_line_col(target.span.end);
                if ctx.byref_param_names.contains(&name_sym) {
                    // A by-ref parameter write mutates caller-visible state
                    // through the reference — a side effect @pure forbids,
                    // same as a global/static-variable write. Also gate on
                    // param_names: byref_param_names is shared with `global`
                    // declarations (see the write-tracking comment below),
                    // which already have their own, declaration-site-only
                    // ImpureGlobalVariable check — a real byref PARAMETER is
                    // also always in param_names, a plain `global $x;`
                    // never is.
                    if (ctx.is_in_pure_fn || ctx.is_in_immutable_method)
                        && ctx.param_names.contains(&name_sym)
                    {
                        self.emit(
                            IssueKind::ImpureByRefAssignment {
                                variable: name_str.clone(),
                            },
                            Severity::Warning,
                            span,
                        );
                    }
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
                // Value type contributed by every non-keyed atom (TArray/TList/
                // TNonEmptyArray/TNonEmptyList) in the union, merged rather than
                // taking just the first match — a heterogeneous union
                // (`array{a:int}|array<string,string>`) must not drop a
                // co-existing generic alternative's value type.
                let mut non_keyed_value_ty = Type::empty();
                let mut has_non_keyed = false;
                for a in &ty.types {
                    if let Atomic::TArray { value, .. }
                    | Atomic::TList { value }
                    | Atomic::TNonEmptyArray { value, .. }
                    | Atomic::TNonEmptyList { value } = a
                    {
                        non_keyed_value_ty.merge_with(value);
                        has_non_keyed = true;
                    }
                }
                // Destructuring a shape-typed source (`['a' => $a] = $arr` or
                // `[$a, $b] = $arr` against `array{0: int, 1: string}`) should
                // resolve each target's type from the matching per-key
                // property instead of always falling back to `mixed` — the
                // fallback above only covers the plain `TArray`/`TList` shapes.
                let mut next_int_key: i64 = 0;
                for elem in elements.iter() {
                    let key: Option<mir_types::atomic::ArrayKey> = match &elem.key {
                        Some(k) => super::helpers::literal_array_key_of_kind(&k.kind),
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
                            if has_non_keyed {
                                result.merge_with(&non_keyed_value_ty);
                                found_any = true;
                            }
                            found_any.then_some(result)
                        })
                        .unwrap_or_else(Type::mixed);
                    // Each destructured target gets its OWN span, not the
                    // outer destructuring statement's span — `[$this->x,
                    // $this->y] = $vals;` previously passed the same `span`
                    // for every element, so a purity/readonly/immutability
                    // diagnostic on the second+ target collided with the
                    // first's under the issue buffer's (kind, file, line,
                    // col_start) dedup key and was silently discarded, even
                    // though it names a different property.
                    self.assign_to_target(&elem.value, elem_ty, ctx, elem.value.span);
                }
            }
            ExprKind::PropertyAccess(pa) => {
                self.check_property_write_purity(pa, ctx, span);
                let obj_ty = self.analyze(&pa.object, ctx);
                // A self/static/parent-typed receiver (e.g. a `self $x` param)
                // previously matched no arm at all below (only TNamedObject),
                // so a write through it got zero property-type/readonly
                // checking. Rebind to a plain TNamedObject using the atom's
                // own already-resolved fqcn so the existing checks apply.
                let obj_ty = crate::expr::objects::rebind_self_static_parent_atom_only(obj_ty);
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
                            // Cross-class immutable write is now checked once,
                            // for every write shape, by `check_property_write_purity`
                            // (called at the top of this arm) — no longer
                            // duplicated here.
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
                                std::sync::Arc::from(format!("prop:{}::{}", prop_owner, prop_name)),
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
                                let in_declaring_scope =
                                    ctx.self_fqcn.as_deref().is_some_and(|self_cls| {
                                        self_cls.eq_ignore_ascii_case(fqcn.as_ref())
                                            && crate::db::property_in_own_composition(
                                                self.db,
                                                crate::db::Fqcn::new(self.db, *fqcn),
                                                &prop_name,
                                            )
                                    });
                                let in_allowed_readonly_scope = (has_native_readonly
                                    || ctx.inside_constructor)
                                    && in_declaring_scope;
                                if is_readonly && !in_allowed_readonly_scope {
                                    self.emit(
                                        IssueKind::ReadonlyPropertyAssignment {
                                            class: prop_owner.to_string(),
                                            property: prop_name.clone(),
                                        },
                                        Severity::Error,
                                        span,
                                    );
                                } else if is_readonly && in_allowed_readonly_scope {
                                    // A second write to the same readonly property within
                                    // the scope PHP otherwise allows initializing it is
                                    // still a runtime error ("cannot modify readonly
                                    // property ... once initialized") — only the FIRST
                                    // write in that scope is legal.
                                    if let ExprKind::Variable(obj_var) = &pa.object.kind {
                                        if ctx.is_readonly_initialized(obj_var.as_ref(), &prop_name)
                                        {
                                            self.emit(
                                                IssueKind::ReadonlyPropertyAlreadyInitialized {
                                                    class: prop_owner.to_string(),
                                                    property: prop_name.clone(),
                                                },
                                                Severity::Error,
                                                span,
                                            );
                                        } else {
                                            ctx.mark_readonly_initialized(
                                                obj_var.as_ref(),
                                                &prop_name,
                                            );
                                        }
                                    }
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
                                        let class_tps = crate::db::class_template_params(
                                            self.db,
                                            fqcn.as_ref(),
                                        )
                                        .map(|tps| tps.to_vec())
                                        .unwrap_or_default();
                                        let mut bindings = crate::generic::build_class_bindings(
                                            &class_tps,
                                            type_params,
                                        );
                                        let inherited_bindings =
                                            crate::db::inherited_template_bindings(
                                                self.db,
                                                fqcn.as_ref(),
                                                &bindings,
                                            );
                                        // Own-bindings-wins only when the
                                        // property is declared directly on
                                        // the receiver's own class
                                        // (`prop_owner`); otherwise the
                                        // ancestor that actually declares it
                                        // wins — same collision guard as the
                                        // read-side property access already
                                        // applies.
                                        if prop_owner.as_ref() == fqcn.as_ref() {
                                            for (k, v) in inherited_bindings {
                                                bindings.entry(k).or_insert(v);
                                            }
                                        } else {
                                            bindings.extend(inherited_bindings);
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
                        // Constructor definite-assignment tracking: a plain
                        // `$this->prop = value` counts as initializing the
                        // property regardless of whether the assigned type is
                        // itself valid (an incompatible assignment is already
                        // flagged separately above; it still runs at runtime).
                        if ctx.inside_constructor && obj_var.as_ref() == "this" {
                            ctx.mark_this_prop_assigned(&prop_name);
                        }
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
                            // ones through a specific receiver. @mutation-free
                            // forbids it too, same as @pure.
                            if ctx.is_in_pure_fn || ctx.is_in_immutable_method {
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
                            let prop_owner =
                                crate::db::find_property_in_chain(self.db, here, prop_name)
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
                            if let Some((owner_cls, prop_def)) =
                                crate::db::find_property_in_chain(self.db, here, &prop_name)
                            {
                                // A `@readonly`-tagged static property has no constructor-
                                // scoped "first write" the way an instance property does —
                                // PHP itself doesn't allow a native `readonly` keyword on a
                                // static property at all, so the docblock-only annotation's
                                // only sensible semantics is "never legal to write from
                                // outside its own declaration", unconditionally.
                                if prop_def.is_readonly {
                                    self.emit(
                                        IssueKind::ReadonlyPropertyAssignment {
                                            class: owner_cls.to_string(),
                                            property: prop_name.clone(),
                                        },
                                        Severity::Error,
                                        span,
                                    );
                                }
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
                let outer_key: Option<Type> = aa
                    .index
                    .as_ref()
                    .map(|idx| super::helpers::coerce_array_key_type(&self.analyze(idx, ctx)));
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
                            // Purity check: `$GLOBALS['x'] = …` / `$_SESSION['x']
                            // = …` reach the same external mutable state as
                            // `global $x;` — mirrors the read-side check in
                            // expr/arrays.rs::analyze_array_access, which this
                            // write path had no equivalent of at all.
                            if (ctx.is_in_pure_fn || ctx.is_in_immutable_method)
                                && crate::util::is_superglobal_name(name_str)
                            {
                                self.emit(
                                    IssueKind::ImpureGlobalVariable {
                                        variable: literal_key_chain
                                            .last()
                                            .and_then(|k| k.as_ref())
                                            .map(|k| match k {
                                                mir_types::atomic::ArrayKey::String(s) => {
                                                    s.to_string()
                                                }
                                                mir_types::atomic::ArrayKey::Int(i) => {
                                                    i.to_string()
                                                }
                                            })
                                            .unwrap_or_else(|| name_str.to_string()),
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                            // `$items['k'] = …` on a by-ref PARAMETER mutates
                            // caller-visible state through the reference,
                            // same as a plain `$items = …` overwrite (which
                            // `assign_to_target`'s own `Variable` arm already
                            // catches) — this array-index-write arm had no
                            // counterpart at all.
                            if (ctx.is_in_pure_fn || ctx.is_in_immutable_method)
                                && ctx
                                    .byref_param_names
                                    .contains(&mir_types::Name::from(name_str))
                                && ctx.param_names.contains(&mir_types::Name::from(name_str))
                            {
                                self.emit(
                                    IssueKind::ImpureByRefAssignment {
                                        variable: name_str.to_string(),
                                    },
                                    Severity::Warning,
                                    span,
                                );
                            }
                            // Base key: innermost index in the chain (closest to $arr).
                            let base_key_opt = key_chain.last().unwrap().clone();
                            let base_key = base_key_opt.unwrap_or_else(Type::mixed);
                            // Only a single-level write ($arr[<key>] = $val, no
                            // nested chain) has a directly-known literal key —
                            // used to update just that one shape property
                            // in place instead of widening the whole shape.
                            // Reuses the same `literal_array_key_of_kind` resolution
                            // already computed for `literal_key_chain` above, rather
                            // than re-deriving it here (a prior duplicate match only
                            // handled string/int keys, missing bool/float/null).
                            let literal_key: Option<mir_types::ArrayKey> = if key_chain.len() == 1 {
                                literal_key_chain.last().cloned().flatten()
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
                                // `$obj[$k] = $v` on an ArrayAccess-implementing receiver
                                // calls offsetSet($k, $v) — the object's own tracked type
                                // doesn't change (unlike a plain array widening), and the
                                // assigned value must satisfy offsetSet's own declared
                                // parameter type instead of falling into the plain-PHP-
                                // array shape-widening logic below, which doesn't apply
                                // to it at all.
                                let array_access_only = !current.is_mixed()
                                    && !current.types.is_empty()
                                    && current.types.iter().all(|a| {
                                        matches!(a, Atomic::TNamedObject { fqcn, .. }
                                            if crate::expr::arrays::implements_array_access(self.db, fqcn))
                                    });
                                if array_access_only && key_chain.len() == 1 {
                                    for atomic in &current.types {
                                        if let Atomic::TNamedObject { fqcn, type_params } = atomic {
                                            if let Some(expected) =
                                                crate::expr::arrays::resolve_array_access_offset_set_value_type(
                                                    self.db, fqcn, type_params,
                                                )
                                            {
                                                if !expected.is_mixed()
                                                    && !property_assign_compatible(
                                                        &ty, &expected, self.db,
                                                    )
                                                {
                                                    self.emit(
                                                        IssueKind::InvalidArgument {
                                                            param: "value".to_string(),
                                                            fn_name: "offsetSet".to_string(),
                                                            expected: expected.to_string(),
                                                            actual: ty.to_string(),
                                                        },
                                                        Severity::Error,
                                                        span,
                                                    );
                                                }
                                            }
                                        }
                                    }
                                    break;
                                }
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
                                let declared_ceiling =
                                    ctx.declared_var_types.get(&mir_types::Name::from(name_str));
                                let updated = match nested_update {
                                    Some(updated) => updated,
                                    None => match &key_chain.last().unwrap() {
                                        None => widen_array_as_list(
                                            &current,
                                            &wrapped_value,
                                            ctx.inside_loop,
                                            declared_ceiling,
                                        ),
                                        Some(_) => widen_array_with_value_and_key(
                                            &current,
                                            &wrapped_value,
                                            &base_key,
                                            literal_key.as_ref(),
                                            ctx.inside_loop,
                                            declared_ceiling,
                                        ),
                                    },
                                };
                                ctx.set_var(name_str, updated);
                            }
                            break;
                        }
                        ExprKind::ArrayAccess(inner) => {
                            // Coerce to PHP's canonical array-key form (bool ->
                            // 0/1, float truncates, null -> ""), same as
                            // `outer_key` above — otherwise a nested write whose
                            // OUTER index is dynamic falls through to this raw,
                            // uncoerced type for every key but the innermost.
                            let inner_key: Option<Type> = inner.index.as_ref().map(|idx| {
                                super::helpers::coerce_array_key_type(&self.analyze(idx, ctx))
                            });
                            literal_key_chain.push(inner.index.as_ref().and_then(|idx| {
                                super::helpers::literal_array_key_of_kind(&idx.kind)
                            }));
                            key_chain.push(inner_key);
                            base = &inner.array;
                        }
                        ExprKind::PropertyAccess(pa) => {
                            // `$this->items[$k] = …` / `$param->items[] = …`:
                            // an array-index write through a property base is
                            // still a mutation of that property (it changes
                            // the array's contents in place), so it must go
                            // through the same purity/immutability checks as
                            // a plain `$obj->items = …` assignment — not just
                            // be read for reference-recording, which is all
                            // this arm previously did.
                            self.check_property_write_purity(pa, ctx, span);
                            self.check_property_readonly_write(pa, ctx, span);
                            let _ = self.analyze(base, ctx);
                            break;
                        }
                        ExprKind::StaticPropertyAccess(spa) => {
                            // `self::$items[$k] = …` / `Foo::$items[] = …`:
                            // same mutation-through-index-write reasoning as
                            // the instance-property arm above, mirroring the
                            // plain `self::$items = …` static-write check
                            // (there's no immutable-context mirror for
                            // statics — see the plain write arm's own comment).
                            if let ExprKind::Identifier(id) = &spa.class.kind {
                                let resolved =
                                    crate::db::resolve_name(self.db, &self.file, id.as_ref());
                                let fqcn_opt: Option<std::sync::Arc<str>> = match resolved.as_str()
                                {
                                    "self" | "static" => {
                                        ctx.self_fqcn.clone().or_else(|| ctx.static_fqcn.clone())
                                    }
                                    "parent" => ctx.parent_fqcn.clone(),
                                    s => Some(std::sync::Arc::from(s)),
                                };
                                if let Some(fqcn) = fqcn_opt {
                                    if let Some(prop_name) = match &spa.member.kind {
                                        ExprKind::Variable(name) | ExprKind::Identifier(name) => {
                                            Some(name.trim_start_matches('$').to_string())
                                        }
                                        _ => None,
                                    } {
                                        if ctx.is_in_pure_fn || ctx.is_in_immutable_method {
                                            self.emit(
                                                IssueKind::ImpureStaticPropertyAssignment {
                                                    class: fqcn.to_string(),
                                                    property: prop_name.clone(),
                                                },
                                                Severity::Warning,
                                                span,
                                            );
                                        }
                                        // `self::$store['k'] = …` mutates the array
                                        // the same way a plain `self::$store = …`
                                        // write would — reuse the same is_readonly
                                        // lookup the plain static-write arm above
                                        // already does, unconditionally (readonly
                                        // is a class contract, not scoped to
                                        // @pure-annotated functions).
                                        if let Some((owner_cls, prop_def)) =
                                            crate::db::find_property_in_chain(
                                                self.db,
                                                crate::db::Fqcn::from_str(self.db, fqcn.as_ref()),
                                                &prop_name,
                                            )
                                        {
                                            if prop_def.is_readonly {
                                                self.emit(
                                                    IssueKind::ReadonlyPropertyAssignment {
                                                        class: owner_cls.to_string(),
                                                        property: prop_name,
                                                    },
                                                    Severity::Error,
                                                    span,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            let _ = self.analyze(base, ctx);
                            break;
                        }
                        _ => {
                            // Non-variable base: analyze it as a read so any
                            // nested property access records its reference.
                            let _ = self.analyze(base, ctx);
                            break;
                        }
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
