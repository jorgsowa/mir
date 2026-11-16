//! The constant value of an argument a security check reads: a literal, or a variable or
//! constant whose type pins one value.
use std::sync::Arc;

use mir_types::{ArrayKey, Atomic, Type};
use php_ast::owned::{Expr, ExprKind};

use crate::ast_ext::ByteStringExt as _;
use crate::db::MirDatabase;
use crate::flow_state::FlowState;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Str(Arc<str>),
    Int(i64),
    Bool(bool),
    Null,
    /// Entries in order; a key or value that isn't constant is `None`.
    Array(Vec<(Option<Value>, Option<Value>)>),
}

impl Value {
    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub(crate) fn entries(&self) -> &[(Option<Value>, Option<Value>)] {
        match self {
            Value::Array(entries) => entries,
            _ => &[],
        }
    }

    /// The entry under string key `key`: `Some(None)` when the key is there with a value that
    /// isn't constant.
    pub(crate) fn entry(&self, key: &str) -> Option<Option<&Value>> {
        self.entries()
            .iter()
            .find(|(k, _)| k.as_ref().and_then(Value::as_str) == Some(key))
            .map(|(_, v)| v.as_ref())
    }
}

/// Resolves expressions against the flow state and declared constants at one point.
pub(crate) struct Values<'a> {
    pub(crate) ctx: &'a FlowState,
    pub(crate) db: &'a dyn MirDatabase,
    pub(crate) file: &'a str,
}

impl Values<'_> {
    pub(crate) fn of(&self, expr: &Expr) -> Option<Value> {
        use php_ast::ast::{BinaryOp, UnaryPrefixOp};
        match &expr.kind {
            ExprKind::String(s) => Some(Value::Str(Arc::from(s.as_str()))),
            ExprKind::Int(i) => Some(Value::Int(*i)),
            ExprKind::Bool(b) => Some(Value::Bool(*b)),
            ExprKind::Null => Some(Value::Null),
            ExprKind::Parenthesized(inner) => self.of(inner),
            ExprKind::UnaryPrefix(u) if u.op == UnaryPrefixOp::Negate => {
                Some(Value::Int(self.of(&u.operand)?.as_int()?.checked_neg()?))
            }
            ExprKind::Binary(b) if b.op == BinaryOp::BitwiseOr => Some(Value::Int(
                self.of(&b.left)?.as_int()? | self.of(&b.right)?.as_int()?,
            )),
            ExprKind::Binary(b) if b.op == BinaryOp::Concat => {
                let (left, right) = (self.of(&b.left)?, self.of(&b.right)?);
                Some(Value::Str(Arc::from(format!(
                    "{}{}",
                    left.as_str()?,
                    right.as_str()?
                ))))
            }
            ExprKind::Array(entries) => {
                if entries.iter().any(|e| e.is_spread() || e.is_by_ref()) {
                    return None;
                }
                Some(Value::Array(
                    entries
                        .iter()
                        .map(|e| (e.key.as_ref().and_then(|k| self.of(k)), self.of(e.value())))
                        .collect(),
                ))
            }
            ExprKind::Variable(name) if self.ctx.var_is_defined(name) => {
                type_value(&self.ctx.get_var(name))
            }
            ExprKind::Name(_) | ExprKind::ClassConstAccess(_) => type_value(
                &crate::expr::constant_expr_type(expr, self.ctx, self.db, self.file)?,
            ),
            _ => None,
        }
    }
}

fn type_value(ty: &Type) -> Option<Value> {
    let [atomic] = ty.types.as_slice() else {
        return None;
    };
    match atomic {
        Atomic::TLiteralString(s) => Some(Value::Str(s.clone())),
        Atomic::TLiteralInt(i) => Some(Value::Int(*i)),
        Atomic::TTrue => Some(Value::Bool(true)),
        Atomic::TFalse => Some(Value::Bool(false)),
        Atomic::TNull => Some(Value::Null),
        Atomic::TKeyedArray { properties, .. } => Some(Value::Array(
            properties
                .iter()
                .map(|(key, prop)| {
                    let key = match key {
                        ArrayKey::String(s) => Value::Str(s.clone()),
                        ArrayKey::Int(i) => Value::Int(*i),
                    };
                    (Some(key), type_value(&prop.ty))
                })
                .collect(),
        )),
        _ => None,
    }
}
