use php_ast::Span;

use mir_codebase::definitions::TemplateParam;
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

    let template_exempt =
        super::param_contains_template_or_unknown(param_ty, arg_ty, ea, template_params)
            && !stub_template_bounds_exclude_null(ea, param_ty);
    if !param_ty.is_nullable()
        && !param_ty.is_mixed()
        && !template_exempt
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
        && !template_exempt
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

/// A stub class template bounded by a non-null type, whose violations `new` doesn't report.
fn stub_template_bounds_exclude_null(ea: &ExpressionAnalyzer<'_>, param_ty: &Type) -> bool {
    let mut any_template = false;
    for a in &param_ty.types {
        let Atomic::TTemplateParam {
            name,
            as_type,
            defining_entity,
        } = a
        else {
            continue;
        };
        if as_type.is_mixed() || as_type.is_nullable() {
            return false;
        }
        let is_class_template = crate::db::class_template_params(ea.db, defining_entity.as_ref())
            .is_some_and(|tps| tps.iter().any(|tp| tp.name == *name));
        let declared_in_stub = is_class_template
            && crate::db::class_like_decl_file(
                ea.db,
                crate::db::Fqcn::from_str(ea.db, defining_entity.as_ref()),
            )
            .is_some_and(|f| crate::stubs::StubVfs::new().is_stub_file(f.as_ref()));
        if !declared_in_stub {
            return false;
        }
        any_template = true;
    }
    any_template
}
