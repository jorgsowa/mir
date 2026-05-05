use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_types::{Atomic, Union};
use php_ast::ast::{CastKind, Expr};

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_cast<'arena, 'src>(
        &mut self,
        kind: &CastKind,
        inner: &Expr<'arena, 'src>,
        ctx: &mut Context,
    ) -> Union {
        let _inner_ty = self.analyze(inner, ctx);
        match kind {
            CastKind::Int => Union::single(Atomic::TInt),
            CastKind::Float => Union::single(Atomic::TFloat),
            CastKind::String => Union::single(Atomic::TString),
            CastKind::Bool => Union::single(Atomic::TBool),
            CastKind::Array => Union::single(Atomic::TArray {
                key: Box::new(Union::single(Atomic::TMixed)),
                value: Box::new(Union::mixed()),
            }),
            CastKind::Object => Union::single(Atomic::TObject),
            CastKind::Unset | CastKind::Void => Union::single(Atomic::TNull),
        }
    }
}
