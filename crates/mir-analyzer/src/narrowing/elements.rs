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
    let refined = ctx.get_prop_refined(&receiver, &key).cloned();
    let present = refined.is_some();
    let current = match refined {
        Some(refined) => refined,
        None => {
            // Shapes track their keys already; only generic arrays and lists are handled here.
            let base = prop_receiver_type(ctx, &receiver, db, file);
            let generic = !base.types.is_empty()
                && base.types.iter().all(|a| {
                    matches!(
                        a,
                        Atomic::TArray { .. }
                            | Atomic::TNonEmptyArray { .. }
                            | Atomic::TList { .. }
                            | Atomic::TNonEmptyList { .. }
                    )
                });
            match mir_types::union::eval_value_of_type(&base).filter(|_| generic) {
                Some(value) => value,
                None => return,
            }
        }
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
    let narrowed = ctx.get_var(STAND_IN);
    ctx.unset_var(STAND_IN);
    ctx.diverges = diverges;
    if !narrowed.is_empty() && (present || !narrowed.is_nullable()) && narrowed != current {
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
