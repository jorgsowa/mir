use super::*;

/// Properties read through `$this->name`, in source order.
pub(super) struct ThisPropReads(pub(super) Vec<mir_types::Name>);

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
    if !ctx.param_parts.is_empty() {
        ctx.param_parts = Default::default();
    }
    ctx.object_guard_slots = None;
    for (index, name) in names.take(MAX_TRACKED_PARAMS).enumerate() {
        ctx.set_var_taint(name, Taint::param(index));
        ctx.set_param_parts(name, Taint::param(index));
    }
}

/// Hand the label slots parameters and `$this`'s properties leave free to the
/// parameters the body proves objects (with `instanceof`, or a callee's).
pub fn seed_object_guards(
    ctx: &mut FlowState,
    params: &[mir_codebase::DeclaredParam],
    prop_labels: &[(usize, mir_types::Name)],
    body: &[php_ast::owned::Stmt],
) {
    if !ctx.summarizing_taint {
        return;
    }
    let params = named_params(params);
    let tracked = params.len().min(MAX_TRACKED_PARAMS);
    let mut free: Vec<usize> = (tracked..MAX_TRACKED_PARAMS)
        .filter(|slot| !prop_labels.iter().any(|(s, _)| s == slot))
        .collect();
    // Callees' guards claim slots as they come; parameters this body tests
    // itself get theirs first.
    let tested = tested_variables(body);
    let assigned = (0..tracked)
        .filter(|i| {
            tested
                .iter()
                .any(|v| v.as_str() == params[*i].name.as_ref())
        })
        .zip(std::iter::from_fn(|| {
            (!free.is_empty()).then(|| free.remove(0))
        }))
        .collect();
    ctx.object_guard_slots = Some(std::sync::Arc::new(std::sync::Mutex::new(
        crate::flow_state::ObjectGuardSlots { free, assigned },
    )));
}

/// Variables tested with `instanceof`, and those whose values reach them by
/// assignment or iteration.
pub(super) fn tested_variables(body: &[php_ast::owned::Stmt]) -> Vec<mir_types::Name> {
    let mut scan = ObjectTests::default();
    for stmt in body {
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut scan, stmt);
    }
    let mut tested = scan.subjects;
    let mut index = 0;
    while let Some(var) = tested.get(index).copied() {
        for (target, sources) in &scan.copies {
            if *target == var {
                for source in sources {
                    if !tested.contains(source) {
                        tested.push(*source);
                    }
                }
            }
        }
        index += 1;
    }
    tested
}

#[derive(Default)]
pub(super) struct ObjectTests {
    subjects: Vec<mir_types::Name>,
    /// Each assigned variable and the variables its value is built from.
    copies: Vec<(mir_types::Name, Vec<mir_types::Name>)>,
}

impl ObjectTests {
    fn copy(&mut self, target: &Expr, value: &Expr) {
        let mut sources = VariableReads(Vec::new());
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_expr(&mut sources, value);
        let mut targets = VariableReads(Vec::new());
        let _ = php_ast::owned::visitor::OwnedVisitor::visit_expr(&mut targets, target);
        for target in targets.0 {
            self.copies.push((target, sources.0.clone()));
        }
    }
}

impl php_ast::owned::visitor::OwnedVisitor for ObjectTests {
    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        if let php_ast::owned::StmtKind::Foreach(fe) = &stmt.kind {
            self.copy(&fe.value, &fe.expr);
        }
        crate::recursion::walk_stmt(self, stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
        match &expr.kind {
            ExprKind::Binary(b) if b.op == php_ast::ast::BinaryOp::Instanceof => {
                if let ExprKind::Variable(v) = &unwrap_parens(&b.left).kind {
                    let name = mir_types::Name::from(v.trim_start_matches('$'));
                    if !self.subjects.contains(&name) {
                        self.subjects.push(name);
                    }
                }
            }
            ExprKind::Assign(a) => self.copy(&a.target, &a.value),
            _ => {}
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// The variable a condition proves an object when it is `is_true`:
/// `$x instanceof C`, or `$x instanceof A || $x instanceof B`.
pub fn proven_object(expr: &Expr, is_true: bool) -> Option<&str> {
    use php_ast::ast::{BinaryOp, UnaryPrefixOp};
    match &expr.kind {
        ExprKind::Parenthesized(inner) => proven_object(inner, is_true),
        ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::BooleanNot => {
            proven_object(&u.operand, !is_true)
        }
        ExprKind::Binary(b) if is_true && b.op == BinaryOp::Instanceof => {
            match &unwrap_parens(&b.left).kind {
                ExprKind::Variable(v) => Some(v.trim_start_matches('$')),
                _ => None,
            }
        }
        ExprKind::Binary(b)
            if is_true && matches!(b.op, BinaryOp::BooleanOr | BinaryOp::LogicalOr) =>
        {
            let left = proven_object(&b.left, true)?;
            (proven_object(&b.right, true) == Some(left)).then_some(left)
        }
        _ => None,
    }
}

/// `var` holds an object here: each parameter whose own value it holds now
/// flows only as that parameter's object guard.
pub fn guard_object(ctx: &mut FlowState, var: &str) {
    let parts = ctx.param_parts(var);
    if parts.is_empty() {
        return;
    }
    let guards = object_guard_slots(ctx.var_taint(var), parts, ctx);
    ctx.map_var_taint(var, |taint| as_object(taint, &guards));
    ctx.set_param_parts(var, parts);
}

/// `(param, slot)` for each parameter `taint` carries `parts` of, claiming its
/// object guard slot.
pub(super) fn object_guard_slots(
    taint: Taint,
    parts: Taint,
    ctx: &FlowState,
) -> Vec<(usize, usize)> {
    (0..MAX_TRACKED_PARAMS)
        .filter(|param| {
            parts.intersects(Taint::param(*param)) && !taint.param_kinds(*param).is_empty()
        })
        .filter_map(|param| Some((param, ctx.object_guard_slot(param)?)))
        .collect()
}

/// `taint` of a value known to be an object: each parameter in `guards` flows
/// as its object guard instead.
pub(super) fn as_object(taint: Taint, guards: &[(usize, usize)]) -> Taint {
    guards.iter().fold(taint, |out, (param, slot)| {
        out.without_label(*param) | Taint::label(*slot, out.param_kinds(*param))
    })
}

/// Taint an argument passes into a callee's object guard: when it holds
/// (part of) a parameter of the body being summarized, that only flows if the
/// parameter holds an object too.
pub(super) fn object_arg_taint(
    value: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    let taint = expr_taint(value, ctx, db, file);
    let guards = object_guard_slots(taint, param_parts(value, ctx, db, file), ctx);
    as_object(taint, &guards)
}

/// Label slots `expr` carries only as (part of) an argument itself: the
/// parameter, a copy, or its elements, never something computed from it.
pub fn param_parts(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Taint {
    if !ctx.summarizing_taint {
        return Taint::NONE;
    }
    let (parts, computed) = split_param_parts(expr, ctx, db, file);
    parts.difference(computed)
}

/// Label slots `expr` carries as (part of) an argument, and those it carries
/// through anything else.
pub(super) fn split_param_parts(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> (Taint, Taint) {
    let either = |exprs: &[&Expr]| {
        exprs.iter().fold((Taint::NONE, Taint::NONE), |(p, c), e| {
            let (ep, ec) = split_param_parts(e, ctx, db, file);
            (p | ep, c | ec)
        })
    };
    match &expr.kind {
        ExprKind::Variable(name) => {
            let parts = ctx.param_parts(name);
            let slots = expr_taint(expr, ctx, db, file).label_slots();
            (parts.intersection(slots), slots.difference(parts))
        }
        ExprKind::Parenthesized(inner) => split_param_parts(inner, ctx, db, file),
        ExprKind::ArrayAccess(aa) => {
            let (parts, computed) = split_param_parts(&aa.array, ctx, db, file);
            let slots = expr_taint(expr, ctx, db, file).label_slots();
            (parts.intersection(slots), computed.intersection(slots))
        }
        ExprKind::Ternary(t) => {
            either(&[t.then_expr.as_deref().unwrap_or(&t.condition), &t.else_expr])
        }
        ExprKind::NullCoalesce(nc) => either(&[&nc.left, &nc.right]),
        ExprKind::Array(elements) => either(
            &elements
                .iter()
                .flat_map(|el| el.key.iter().chain(std::iter::once(&el.value)))
                .collect::<Vec<_>>(),
        ),
        ExprKind::FunctionCall(fc) => match element_preserving_args(&fc.name, &fc.args, db, file) {
            Some(arrays) => either(&arrays),
            None => split_call_parts(expr, ctx, db, file),
        },
        ExprKind::MethodCall(_)
        | ExprKind::NullsafeMethodCall(_)
        | ExprKind::StaticMethodCall(_) => split_call_parts(expr, ctx, db, file),
        _ => (Taint::NONE, expr_taint(expr, ctx, db, file).label_slots()),
    }
}

/// [`split_param_parts`] of a call: arguments a summarized callee returns
/// unchanged keep their parts.
pub(super) fn split_call_parts(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> (Taint, Taint) {
    let slots = expr_taint(expr, ctx, db, file).label_slots();
    if slots.is_empty() {
        return (Taint::NONE, Taint::NONE);
    }
    let Some((summary, params, args)) = summarized_callee(expr, ctx, db, file) else {
        return (Taint::NONE, slots);
    };
    let (mut parts, mut computed) = (Taint::NONE, Taint::NONE);
    for slot in 0..MAX_TRACKED_PARAMS {
        if summary.returns.param_kinds(slot).is_empty() || summary.prop_label(slot).is_some() {
            continue;
        }
        let unchanged =
            summary.return_parts.intersects(Taint::param(slot)) && !summary.is_object_guard(slot);
        for param in summary.slot_params(slot) {
            for arg in param_arg_exprs(&params, args, param) {
                let (p, c) = split_param_parts(arg, ctx, db, file);
                if unchanged {
                    parts |= p;
                    computed |= c;
                } else {
                    computed |= p | c;
                }
            }
        }
    }
    // The receiver's own taint may flow into the result as well.
    if let ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) = &expr.kind {
        computed |= expr_taint(&mc.object, ctx, db, file).label_slots();
    }
    let parts = parts.difference(computed);
    (parts, slots.difference(parts))
}

/// A callee's summary and parameters, with the arguments a call passes.
pub(super) type SummarizedCall<'e> = (
    TaintSummary,
    std::sync::Arc<[mir_codebase::DeclaredParam]>,
    &'e [php_ast::owned::Arg],
);

/// The call whose single target is a summarized user function or method
/// without taint annotations.
pub(super) fn summarized_callee<'e>(
    expr: &'e Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<SummarizedCall<'e>> {
    let method = |class: &str, name: &Expr, args: &'e [php_ast::owned::Arg]| {
        let ExprKind::Identifier(name) = &name.kind else {
            return None;
        };
        let lower = crate::util::php_ident_lowercase(name);
        let (owner, m) = crate::db::find_method_respecting_precedence(
            db,
            crate::db::Fqcn::from_str(db, class),
            &lower,
        )?;
        if m.is_abstract
            || m.taint.is_some()
            || crate::db::is_interface(db, owner.as_ref())
            || CalleeBody::of(m.location.as_ref(), file) != CalleeBody::Summarized
            || !crate::db::taint_sources::method_returns_param(db, file, owner.as_ref(), &lower)
        {
            return None;
        }
        let summary = crate::db::method_taint_summary(db, file, owner.as_ref(), &lower)?;
        Some((summary, m.params.clone(), args))
    };
    match &expr.kind {
        ExprKind::FunctionCall(fc) => {
            let ExprKind::Identifier(name) = &fc.name.kind else {
                return None;
            };
            let resolved = crate::db::resolve_name(db, file, name);
            let f =
                crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved))
                    .or_else(|| {
                    (!name.contains('\\'))
                        .then(|| {
                            crate::db::find_function_from(
                                db,
                                file,
                                crate::db::Fqcn::from_str(db, name),
                            )
                        })
                        .flatten()
                })?;
            if f.taint.is_some()
                || CalleeBody::of(f.location.as_ref(), file) != CalleeBody::Summarized
                || !crate::db::taint_sources::function_returns_param(db, file, f.fqn.as_ref())
            {
                return None;
            }
            let summary = crate::db::function_taint_summary(db, file, f.fqn.as_ref())?;
            Some((summary, f.params.clone(), &fc.args[..]))
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            match receiver_classes(&mc.object, ctx, db, file).as_slice() {
                [class] => method(class, &mc.method, &mc.args),
                _ => None,
            }
        }
        ExprKind::StaticMethodCall(smc) => {
            let ExprKind::Identifier(class) = &smc.class.kind else {
                return None;
            };
            let class = match crate::db::resolve_name(db, file, class).as_str() {
                "self" | "static" => ctx.self_fqcn.clone()?,
                "parent" => ctx.parent_fqcn.clone()?,
                resolved => std::sync::Arc::from(resolved),
            };
            method(&class, &smc.method, &smc.args)
        }
        _ => None,
    }
}

/// The arrays a builtin's result takes its elements from unchanged
/// (`array_filter($xs, $f)`, `array_merge($a, $b)`, `reset($xs)`).
pub(super) fn element_preserving_args<'e>(
    callee: &Expr,
    args: &'e [php_ast::owned::Arg],
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> Option<Vec<&'e Expr>> {
    let ExprKind::Identifier(name) = &callee.kind else {
        return None;
    };
    let global = name.trim_start_matches('\\');
    if global.contains('\\')
        || (!name.starts_with('\\')
            && crate::db::find_function_from(
                db,
                file,
                crate::db::Fqcn::from_str(db, &crate::db::resolve_name(db, file, name)),
            )
            .is_some_and(|f| is_user_code(f.location.as_ref())))
    {
        return None;
    }
    let values = |n: usize| -> Vec<&Expr> {
        args.iter()
            .take(n)
            .filter_map(|a| a.value.as_ref())
            .collect()
    };
    match crate::util::php_ident_lowercase(global).as_str() {
        "array_filter"
        | "array_values"
        | "array_reverse"
        | "array_slice"
        | "array_unique"
        | "array_diff"
        | "array_diff_key"
        | "array_diff_assoc"
        | "array_intersect"
        | "array_intersect_key"
        | "array_intersect_assoc"
        | "reset"
        | "end"
        | "current"
        | "pos"
        | "next"
        | "prev"
        | "array_shift"
        | "array_pop" => Some(values(1)),
        "array_merge" => Some(values(args.len())),
        _ => None,
    }
}

/// Whether a value of type `ty` may be, or contain, an object.
pub(super) fn type_may_hold_object(ty: &mir_types::Type) -> bool {
    use mir_types::Atomic;
    ty.types.is_empty()
        || ty.types.iter().any(|atom| match atom {
            Atomic::TString
            | Atomic::TLiteralString(_)
            | Atomic::TCallableString
            | Atomic::TClassString(_)
            | Atomic::TNonEmptyString
            | Atomic::TNumericString
            | Atomic::TInt
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
            | Atomic::TVoid
            | Atomic::TNever
            | Atomic::TScalar
            | Atomic::TNumeric
            | Atomic::TInterfaceString(_)
            | Atomic::TEnumString
            | Atomic::TTraitString => false,
            Atomic::TArray { key: _, value }
            | Atomic::TNonEmptyArray { key: _, value }
            | Atomic::TList { value }
            | Atomic::TNonEmptyList { value } => type_may_hold_object(value),
            Atomic::TKeyedArray {
                properties,
                is_open,
                ..
            } => *is_open || properties.values().any(|p| type_may_hold_object(&p.ty)),
            _ => true,
        })
}

/// Whether `expr`'s value may be, or contain, an object, judged from its shape
/// and declared types; request data never is.
pub fn value_may_hold_object(
    expr: &Expr,
    ctx: &FlowState,
    db: &dyn crate::db::MirDatabase,
    file: &str,
) -> bool {
    use php_ast::ast::BinaryOp;
    let declared = |ty: Option<&mir_types::Type>| ty.is_none_or(type_may_hold_object);
    match &unwrap_parens(expr).kind {
        ExprKind::Variable(name) => {
            let name = name.trim_start_matches('$');
            !is_superglobal(name)
                && !ctx.is_plain(name)
                && declared(ctx.vars.get(&mir_types::Name::from(name)).map(|t| &**t))
        }
        ExprKind::ArrayAccess(aa) => value_may_hold_object(&aa.array, ctx, db, file),
        ExprKind::String(_)
        | ExprKind::Int(_)
        | ExprKind::Float(_)
        | ExprKind::Bool(_)
        | ExprKind::Null
        | ExprKind::InterpolatedString(_)
        | ExprKind::Heredoc { .. }
        | ExprKind::Nowdoc { .. }
        | ExprKind::Isset(_)
        | ExprKind::Empty(_)
        | ExprKind::MagicConst(_) => false,
        ExprKind::Cast(kind, inner) => match kind {
            php_ast::ast::CastKind::Array => value_may_hold_object(inner, ctx, db, file),
            php_ast::ast::CastKind::Object => true,
            _ => false,
        },
        ExprKind::Binary(b) => !matches!(
            b.op,
            BinaryOp::Concat
                | BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Identical
                | BinaryOp::NotIdentical
                | BinaryOp::Less
                | BinaryOp::Greater
                | BinaryOp::LessOrEqual
                | BinaryOp::GreaterOrEqual
                | BinaryOp::Spaceship
                | BinaryOp::BooleanAnd
                | BinaryOp::BooleanOr
                | BinaryOp::LogicalAnd
                | BinaryOp::LogicalOr
                | BinaryOp::LogicalXor
                | BinaryOp::Instanceof
        ),
        ExprKind::Array(elements) => elements
            .iter()
            .any(|el| value_may_hold_object(&el.value, ctx, db, file)),
        ExprKind::Ternary(t) => {
            value_may_hold_object(
                t.then_expr.as_deref().unwrap_or(&t.condition),
                ctx,
                db,
                file,
            ) || value_may_hold_object(&t.else_expr, ctx, db, file)
        }
        ExprKind::NullCoalesce(nc) => {
            value_may_hold_object(&nc.left, ctx, db, file)
                || value_may_hold_object(&nc.right, ctx, db, file)
        }
        ExprKind::FunctionCall(fc) => {
            let ExprKind::Identifier(name) = &fc.name.kind else {
                return true;
            };
            let resolved = crate::db::resolve_name(db, file, name);
            let func =
                crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, &resolved))
                    .or_else(|| {
                        (!name.contains('\\'))
                            .then(|| {
                                crate::db::find_function_from(
                                    db,
                                    file,
                                    crate::db::Fqcn::from_str(db, name),
                                )
                            })
                            .flatten()
                    });
            func.is_none_or(|f| declared(f.return_type.as_deref()))
        }
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            let ExprKind::Identifier(method) = &mc.method.kind else {
                return true;
            };
            let Some(recv) =
                crate::expr::assignment::resolve_chained_receiver_type(&mc.object, ctx, db, file)
            else {
                return true;
            };
            let method = crate::util::php_ident_lowercase(method);
            recv.types.is_empty()
                || recv.types.iter().any(|atom| {
                    let mir_types::Atomic::TNamedObject { fqcn, .. } = atom else {
                        return true;
                    };
                    crate::db::find_method_respecting_precedence(
                        db,
                        crate::db::Fqcn::from_str(db, fqcn.as_ref()),
                        &method,
                    )
                    .is_none_or(|(_, m)| m.return_type.as_deref().is_none_or(type_may_hold_object))
                })
        }
        _ => true,
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
    if !db.taint_analysis() {
        return Taint::NONE;
    }
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
    bound_arg_exprs(
        args,
        index,
        param.name.as_ref(),
        param.is_variadic,
        |name| params.iter().any(|p| p.name.as_ref() == name),
    )
}

/// Expressions passed for the parameter `name` at `index`; `is_param` tells declared names
/// from the extra named arguments a variadic collects. A spread of an array literal binds
/// element by element, any other spread may cover every parameter from its position on.
pub(super) fn bound_arg_exprs<'e>(
    args: &'e [php_ast::owned::Arg],
    index: usize,
    name: &str,
    variadic: bool,
    is_param: impl Fn(&str) -> bool,
) -> Vec<&'e Expr> {
    let named_here = |key: &str| key == name || (variadic && !is_param(key));
    let mut out = Vec::new();
    let mut position = 0;
    for arg in args {
        let Some(value) = &arg.value else {
            position += 1;
            continue;
        };
        if let Some(arg_name) = &arg.name {
            if named_here(&crate::parser::name_to_string_owned(arg_name)) {
                out.push(value);
            }
            continue;
        }
        if !arg.unpack {
            if position == index || (variadic && position >= index) {
                out.push(value);
            }
            position += 1;
            continue;
        }
        let ExprKind::Array(elements) = &unwrap_parens(value).kind else {
            if position <= index || variadic {
                out.push(value);
            }
            position += 1;
            continue;
        };
        // An element that may expand to any number of arguments leaves later positions unknown.
        let mut open_ended = false;
        for el in elements {
            match literal_element_key(el) {
                ElementKey::Named(key) => {
                    if named_here(key) {
                        out.push(&el.value);
                    }
                }
                ElementKey::Positional => {
                    if position == index
                        || (variadic && (open_ended || position >= index))
                        || (open_ended && position <= index)
                    {
                        out.push(&el.value);
                    }
                    position += 1;
                }
                ElementKey::Unknown => {
                    open_ended = true;
                    out.push(&el.value);
                }
            }
        }
    }
    out
}

/// How an array-literal element binds when the array is spread into a call.
pub(super) enum ElementKey<'e> {
    Named(&'e str),
    Positional,
    Unknown,
}

pub(super) fn literal_element_key(el: &php_ast::owned::ArrayElement) -> ElementKey<'_> {
    if el.unpack {
        return ElementKey::Unknown;
    }
    match el.key.as_ref().map(|k| &k.kind) {
        None | Some(ExprKind::Int(_)) => ElementKey::Positional,
        Some(ExprKind::String(key)) => ElementKey::Named(key.as_ref()),
        Some(_) => ElementKey::Unknown,
    }
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
        None => summary
            .slot_args(slot, &ctor.params, &new.args, &[], ctx, db, file)
            .into_iter()
            .fold(Taint::NONE, |acc, e| {
                acc | expr_taint(e, ctx, db, file).concrete()
            }),
    };
    summary
        .prop_writes
        .iter()
        .filter(|(key, _)| !key.as_str().starts_with('$'))
        .map(|(key, taint)| (*key, resolve_labels(*taint, arg_taint)))
        .filter(|(_, taint)| !taint.is_empty())
        .collect()
}
