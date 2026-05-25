use mir_types::{Atomic, Type};
use php_ast::owned::ExprKind;

pub(crate) fn analyze(kind: &ExprKind) -> Type {
    match kind {
        ExprKind::Int(n) => Type::single(Atomic::TLiteralInt(*n)),
        ExprKind::Float(f) => {
            let bits = f.to_bits();
            Type::single(Atomic::TLiteralFloat(
                (bits >> 32) as i64,
                (bits & 0xFFFF_FFFF) as i64,
            ))
        }
        ExprKind::String(s) => Type::single(Atomic::TLiteralString(s.as_ref().into())),
        ExprKind::Bool(b) => {
            if *b {
                Type::single(Atomic::TTrue)
            } else {
                Type::single(Atomic::TFalse)
            }
        }
        ExprKind::Null => Type::single(Atomic::TNull),
        _ => Type::mixed(),
    }
}
