use super::ExpressionAnalyzer;
use crate::context::Context;
use mir_types::{Atomic, Union};
use php_ast::ast::MagicConstKind;
use php_ast::owned::YieldExpr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_yield(&mut self, y: &YieldExpr, ctx: &mut Context) -> Union {
        if let Some(key) = &y.key {
            self.analyze(key, ctx);
        }
        if let Some(value) = &y.value {
            self.analyze(value, ctx);
        }
        Union::mixed()
    }

    pub(super) fn analyze_magic_const(kind: &MagicConstKind) -> Union {
        match kind {
            MagicConstKind::Line => Union::single(Atomic::TInt),
            MagicConstKind::File
            | MagicConstKind::Dir
            | MagicConstKind::Function
            | MagicConstKind::Class
            | MagicConstKind::Method
            | MagicConstKind::Namespace
            | MagicConstKind::Trait
            | MagicConstKind::Property => Union::single(Atomic::TString),
        }
    }
}
