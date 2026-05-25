use super::ExpressionAnalyzer;
use crate::flow_state::FlowState;
use mir_types::{Atomic, Type};
use php_ast::ast::MagicConstKind;
use php_ast::owned::YieldExpr;

impl<'a> ExpressionAnalyzer<'a> {
    pub(super) fn analyze_yield(&mut self, y: &YieldExpr, ctx: &mut FlowState) -> Type {
        if let Some(key) = &y.key {
            self.analyze(key, ctx);
        }
        if let Some(value) = &y.value {
            self.analyze(value, ctx);
        }
        Type::mixed()
    }

    pub(super) fn analyze_magic_const(kind: &MagicConstKind) -> Type {
        match kind {
            MagicConstKind::Line => Type::single(Atomic::TInt),
            MagicConstKind::File
            | MagicConstKind::Dir
            | MagicConstKind::Function
            | MagicConstKind::Class
            | MagicConstKind::Method
            | MagicConstKind::Namespace
            | MagicConstKind::Trait
            | MagicConstKind::Property => Type::single(Atomic::TString),
        }
    }
}
