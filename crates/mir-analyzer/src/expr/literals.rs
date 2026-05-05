use mir_types::{Atomic, Union};
use php_ast::ast::ExprKind;

pub(crate) fn analyze<'arena, 'src>(kind: &ExprKind<'arena, 'src>) -> Union {
    match kind {
        ExprKind::Int(n) => Union::single(Atomic::TLiteralInt(*n)),
        ExprKind::Float(f) => {
            let bits = f.to_bits();
            Union::single(Atomic::TLiteralFloat(
                (bits >> 32) as i64,
                (bits & 0xFFFF_FFFF) as i64,
            ))
        }
        ExprKind::String(s) => Union::single(Atomic::TLiteralString((*s).into())),
        ExprKind::Bool(b) => {
            if *b {
                Union::single(Atomic::TTrue)
            } else {
                Union::single(Atomic::TFalse)
            }
        }
        ExprKind::Null => Union::single(Atomic::TNull),
        _ => Union::mixed(),
    }
}
