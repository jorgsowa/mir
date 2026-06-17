use php_ast::Span;

use mir_codebase::storage::TemplateParam;
use mir_issues::{IssueKind, Severity};
use mir_types::{Atomic, Type};

use crate::expr::ExpressionAnalyzer;

pub(crate) fn check_one(
    ea: &mut ExpressionAnalyzer<'_>,
    fn_name: &str,
    param_name: &str,
    param_ty: &Type,
    arg_ty: &Type,
    arg_span: Span,
    template_params: &[TemplateParam],
) {
    // `mixed` already subsumes `null`, so a `mixed | null` argument is just
    // `mixed`. Such unions reach here un-normalized from type *inference* (e.g.
    // a magic `__get` whose body returns `$attributes[$key]` or `null`), where
    // the docblock parser's `mixed | null → mixed` collapse never ran. Emitting
    // `PossiblyNullArgument`/`NullArgument` here is both redundant with the
    // `MixedArgument` path and less precise than Psalm (which folds null into
    // mixed), so defer entirely to that path.
    if arg_ty.is_mixed() {
        return;
    }

    if !param_ty.is_nullable()
        && !param_ty.is_mixed()
        && !super::param_contains_template_or_unknown(param_ty, arg_ty, ea, template_params)
        && arg_ty.is_single()
        && arg_ty.contains(|t| matches!(t, Atomic::TNull))
    {
        ea.emit(
            IssueKind::NullArgument {
                param: param_name.to_string(),
                fn_name: fn_name.to_string(),
            },
            Severity::Warning,
            arg_span,
        );
    } else if !param_ty.is_nullable()
        && !param_ty.is_mixed()
        && !super::param_contains_template_or_unknown(param_ty, arg_ty, ea, template_params)
        && arg_ty.is_nullable()
    {
        ea.emit(
            IssueKind::PossiblyNullArgument {
                param: param_name.to_string(),
                fn_name: fn_name.to_string(),
            },
            Severity::Info,
            arg_span,
        );
    }
}
