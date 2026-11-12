//! Narrowing of a generic array's element read through a variable or literal
//! key (`$a[$k] instanceof X`, `is_string($this->m['x'])`): the check runs on a
//! stand-in variable and the result is stored under a `[key]` refinement.
use crate::ast_ext::ByteStringExt as _;
use php_ast::ast::BinaryOp;
use php_ast::owned::fold::{fold_owned_expr, OwnedFold};
use php_ast::owned::{Expr, ExprKind};

use mir_types::Atomic;

use crate::db::MirDatabase;
use crate::flow_state::FlowState;

use super::core::{extract_prop_access, peel_parens, prop_receiver_type};

const STAND_IN: &str = "\u{1}element";

/// `prop_refined` key (`receiver`, `[key]`) of an element read like `$a[$k]` or `$this->m['x']`.
pub(crate) fn element_access_key(expr: &Expr) -> Option<(String, String)> {
    let ExprKind::ArrayAccess(aa) = &peel_parens(expr).kind else {
        return None;
    };
    let key = match &peel_parens(aa.index.as_ref()?).kind {
        ExprKind::Variable(var) => format!("[${}]", var.trim_start_matches('$')),
        ExprKind::String(s) => format!("['{}']", s.as_str()),
        ExprKind::Int(n) => format!("[{n}]"),
        _ => return None,
    };
    let receiver = match &peel_parens(&aa.array).kind {
        ExprKind::Variable(var) => var.trim_start_matches('$').to_string(),
        _ => {
            let (obj, prop) = extract_prop_access(&aa.array)?;
            if prop.ends_with("()") {
                return None;
            }
            format!("{obj}->{prop}")
        }
    };
    Some((receiver, key))
}

/// Forgets the element refinements a write to `target` (`$a[$k] = `, `$this->m[] = `) makes stale.
pub(crate) fn forget_element_write(target: &Expr, ctx: &mut FlowState) {
    let mut base = peel_parens(target);
    if !matches!(base.kind, ExprKind::ArrayAccess(_)) {
        return;
    }
    while let ExprKind::ArrayAccess(aa) = &base.kind {
        base = peel_parens(&aa.array);
    }
    let receiver = match &base.kind {
        ExprKind::Variable(var) => var.trim_start_matches('$').to_string(),
        _ => match extract_prop_access(base) {
            Some((obj, prop)) => format!("{obj}->{prop}"),
            None => return,
        },
    };
    ctx.drop_element_refinements(&receiver, false);
}

/// Narrows the element a leaf check tests, when it is read from a generic array.
pub(super) fn narrow_element(
    expr: &Expr,
    ctx: &mut FlowState,
    is_true: bool,
    db: &dyn MirDatabase,
    file: &str,
) {
    let Some(subject) = element_subject(expr) else {
        return;
    };
    let Some((receiver, key)) = element_access_key(subject) else {
        return;
    };
    let Some((current, present)) = element_current_type(ctx, &receiver, &key, db, file) else {
        return;
    };
    // A missing key reads as `null`, so until a check rules `null` out the element may be absent.
    let mut possible = current.clone();
    if !present {
        possible.add_type(Atomic::TNull);
    }
    let folded = StandIn { span: subject.span }.fold_expr(expr.clone());
    let diverges = ctx.diverges;
    ctx.narrow_var(STAND_IN, possible);
    super::narrow_from_condition(&folded, ctx, is_true, db, file);
    let mut narrowed = ctx.get_var(STAND_IN);
    // A passing check the element type rules out means that type is wrong; the check decides.
    if is_true && ctx.diverges && !diverges {
        ctx.diverges = false;
        ctx.narrow_var(STAND_IN, mir_types::Type::mixed());
        super::narrow_from_condition(&folded, ctx, is_true, db, file);
        narrowed = ctx.get_var(STAND_IN);
    }
    ctx.unset_var(STAND_IN);
    ctx.diverges = diverges;
    if !narrowed.is_empty() && (present || !narrowed.is_nullable()) && narrowed != current {
        ctx.set_prop_refined(&receiver, &key, narrowed);
    }
}

/// Current type of the element `receiver[key]` read from a generic array, and whether a
/// refinement already holds it.
fn element_current_type(
    ctx: &mut FlowState,
    receiver: &str,
    key: &str,
    db: &dyn MirDatabase,
    file: &str,
) -> Option<(mir_types::Type, bool)> {
    if let Some(refined) = ctx.get_prop_refined(receiver, key) {
        return Some((refined.clone(), true));
    }
    // A literal key into a shape is narrowed by the shape itself; a variable key
    // may name any of a closed shape's entries.
    let variable_key = key.starts_with("[$");
    // An element read from `null` is `null` too, which the check itself rules in or out.
    let base = prop_receiver_type(ctx, receiver, db, file).remove_null();
    let generic = !base.types.is_empty()
        && base.types.iter().all(|a| match a {
            Atomic::TArray { .. }
            | Atomic::TNonEmptyArray { .. }
            | Atomic::TList { .. }
            | Atomic::TNonEmptyList { .. } => true,
            Atomic::TKeyedArray { is_open, .. } => variable_key && !is_open,
            _ => false,
        });
    let value = mir_types::union::eval_value_of_type(&base).filter(|_| generic)?;
    Some((value, false))
}

/// `isset($a[$k]['x']['y'])` on a generic array: narrows the shape of the element `$a[$k]`
/// along the literal keys that follow it.
pub(crate) fn narrow_isset_element_shape(
    expr: &Expr,
    ctx: &mut FlowState,
    db: &dyn MirDatabase,
    file: &str,
) {
    let mut path = Vec::new();
    let mut element = peel_parens(expr);
    while let ExprKind::ArrayAccess(aa) = &element.kind {
        let key = match aa.index.as_ref().map(|i| &peel_parens(i).kind) {
            Some(ExprKind::String(s)) => {
                mir_types::atomic::ArrayKey::String(std::sync::Arc::from(&*s.to_text()))
            }
            Some(ExprKind::Int(i)) => mir_types::atomic::ArrayKey::Int(*i),
            _ => break,
        };
        path.push(key);
        element = peel_parens(&aa.array);
    }
    if path.is_empty() {
        return;
    }
    path.reverse();
    let Some((receiver, key)) = element_access_key(element) else {
        return;
    };
    let Some((current, _)) = element_current_type(ctx, &receiver, &key, db, file) else {
        return;
    };
    if let Some(narrowed) = super::arrays::shapes::narrow_shape_path(&current, &path) {
        ctx.set_prop_refined(&receiver, &key, narrowed);
    }
}

/// The element a leaf condition tests: the condition itself, an `instanceof`
/// or comparison operand, the sole argument of a call, or a sole `isset()` operand.
fn element_subject(expr: &Expr) -> Option<&Expr> {
    fn element(e: &Expr) -> Option<&Expr> {
        let e = peel_parens(e);
        matches!(e.kind, ExprKind::ArrayAccess(_)).then_some(e)
    }
    match &expr.kind {
        ExprKind::ArrayAccess(_) => Some(expr),
        ExprKind::Binary(b) if b.op == BinaryOp::Instanceof => element(&b.left),
        ExprKind::Binary(b)
            if matches!(
                b.op,
                BinaryOp::Identical | BinaryOp::NotIdentical | BinaryOp::Equal | BinaryOp::NotEqual
            ) =>
        {
            element(&b.left).or_else(|| element(&b.right))
        }
        ExprKind::FunctionCall(call) if call.args.len() == 1 => element(call.args[0].value()?),
        ExprKind::Isset(vars) if vars.len() == 1 => element(&vars[0]),
        _ => None,
    }
}

/// Replaces the node at `span` with the stand-in variable.
struct StandIn {
    span: php_ast::Span,
}

impl OwnedFold for StandIn {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        if expr.span == self.span && matches!(expr.kind, ExprKind::ArrayAccess(_)) {
            return Expr {
                kind: ExprKind::Variable(STAND_IN.into()),
                span: expr.span,
            };
        }
        fold_owned_expr(self, expr)
    }
}
