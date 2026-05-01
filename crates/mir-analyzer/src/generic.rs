/// Generic type inference — infer template bindings from argument types and
/// substitute them into return types.
use std::collections::HashMap;
use std::sync::Arc;

use mir_codebase::storage::{FnParam, TemplateParam};
use mir_types::{Atomic, Union};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Infer template parameter bindings by matching parameter types against
/// argument types.
///
/// For example, given `function identity<T>(T $x): T` called with `"hello"`,
/// this returns `{ T → string }`.
pub fn infer_template_bindings(
    template_params: &[TemplateParam],
    params: &[FnParam],
    arg_types: &[Union],
) -> HashMap<Arc<str>, Union> {
    let mut bindings: HashMap<Arc<str>, Union> = HashMap::new();
    let template_names: std::collections::HashSet<Arc<str>> =
        template_params.iter().map(|tp| tp.name.clone()).collect();

    for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
        if let Some(param_ty) = &param.ty {
            infer_from_pair(param_ty, arg_ty, &template_names, &mut bindings);
        }
    }

    // For any template not bound through arguments, fall back to its bound
    // (or mixed if no bound is declared).
    for tp in template_params {
        bindings
            .entry(tp.name.clone())
            .or_insert_with(|| tp.bound.clone().unwrap_or_else(Union::mixed));
    }

    bindings
}

/// Check that each binding satisfies the template's declared bound.
/// Returns a list of `(template_name, inferred_type, bound)` for violations.
pub fn check_template_bounds<'a>(
    bindings: &'a HashMap<Arc<str>, Union>,
    template_params: &'a [TemplateParam],
) -> Vec<(&'a Arc<str>, &'a Union, &'a Union)> {
    let mut violations = Vec::new();
    for tp in template_params {
        if let Some(bound) = &tp.bound {
            if let Some(inferred) = bindings.get(&tp.name) {
                if !bound.is_mixed()
                    && !inferred.is_mixed()
                    && !inferred.is_subtype_of_simple(bound)
                {
                    violations.push((&tp.name, inferred, bound));
                }
            }
        }
    }
    violations
}

/// Build template bindings from a receiver's concrete type params.
///
/// Zips `class_template_params` (e.g. `[T]` declared on the class) with
/// `receiver_type_params` (e.g. `[User]` from `Collection<User>`) to produce
/// `{ T → User }`. If the receiver supplies fewer type params than the class
/// declares, the trailing template params are left unbound. If the receiver
/// supplies more, the extras are ignored.
pub fn build_class_bindings(
    class_template_params: &[TemplateParam],
    receiver_type_params: &[Union],
) -> HashMap<Arc<str>, Union> {
    class_template_params
        .iter()
        .zip(receiver_type_params.iter())
        .map(|(tp, ty)| (tp.name.clone(), ty.clone()))
        .collect()
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// If `param_ty` is a union mixing template placeholders with concrete atomics,
/// return `arg_ty` with the concrete atomics filtered out — what the template
/// should actually bind to. Returns `None` when no filtering is needed.
fn compute_template_residual(
    param_ty: &Union,
    arg_ty: &Union,
    template_names: &std::collections::HashSet<Arc<str>>,
) -> Option<Union> {
    let mut has_template = false;
    let mut concrete: Vec<&Atomic> = Vec::new();
    for a in &param_ty.types {
        if is_template_atomic(a, template_names) {
            has_template = true;
        } else {
            concrete.push(a);
        }
    }
    if !has_template || concrete.is_empty() {
        return None;
    }
    let mut residual = Union::empty();
    residual.from_docblock = arg_ty.from_docblock;
    residual.possibly_undefined = arg_ty.possibly_undefined;
    for a in &arg_ty.types {
        if !concrete.iter().any(|c| atomics_match_for_filter(c, a)) {
            residual.add_type(a.clone());
        }
    }
    if residual.types.is_empty() || residual.types.len() == arg_ty.types.len() {
        return None;
    }
    Some(residual)
}

fn is_template_atomic(a: &Atomic, template_names: &std::collections::HashSet<Arc<str>>) -> bool {
    match a {
        Atomic::TTemplateParam { .. } => true,
        Atomic::TNamedObject { fqcn, type_params } => {
            type_params.is_empty() && !fqcn.contains('\\') && template_names.contains(fqcn.as_ref())
        }
        _ => false,
    }
}

/// Conservative atomic-kind match for filtering arg atomics out of a residual.
/// Returns true when an arg atomic is "covered" by a concrete param atomic so
/// the template need not absorb it. Only matches the simple kinds we expect to
/// see paired with templates in unions (null, bool, int, string, etc.).
fn atomics_match_for_filter(concrete: &Atomic, arg: &Atomic) -> bool {
    matches!(
        (concrete, arg),
        (Atomic::TNull, Atomic::TNull)
            | (Atomic::TBool, Atomic::TBool)
            | (Atomic::TBool, Atomic::TTrue)
            | (Atomic::TBool, Atomic::TFalse)
            | (Atomic::TTrue, Atomic::TTrue)
            | (Atomic::TFalse, Atomic::TFalse)
            | (Atomic::TInt, Atomic::TInt)
            | (Atomic::TFloat, Atomic::TFloat)
            | (Atomic::TString, Atomic::TString)
    )
}

/// Recursively match `param_ty` (which may contain template placeholders)
/// against `arg_ty` (a concrete type), updating `bindings`.
///
/// `template_names` is the set of template names declared on the surrounding
/// function/method. Bare unqualified `TNamedObject` references whose fqcn is in
/// that set are treated as template-param references — the docblock parser
/// emits them that way because it lacks template context at parse time
/// (mirrors the workaround in `Union::substitute_templates`).
fn infer_from_pair(
    param_ty: &Union,
    arg_ty: &Union,
    template_names: &std::collections::HashSet<Arc<str>>,
    bindings: &mut HashMap<Arc<str>, Union>,
) {
    // When the parameter is a union mixing template placeholders with concrete
    // atomics (e.g. `T|null` against `Bar|null`), the template should bind to
    // the residual after subtracting matching concrete atomics — otherwise
    // `T` ends up as `Bar|null` instead of `Bar`.
    let template_residual = compute_template_residual(param_ty, arg_ty, template_names);

    for p_atomic in &param_ty.types {
        match p_atomic {
            // Direct template placeholder: T → bind T = residual(arg_ty)
            Atomic::TTemplateParam { name, .. } => {
                let bind = template_residual.as_ref().unwrap_or(arg_ty);
                let entry = bindings.entry(name.clone()).or_insert_with(Union::empty);
                *entry = Union::merge(entry, bind);
            }

            // array<K, V> matched against array<k_ty, v_ty>
            Atomic::TArray { key: pk, value: pv } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TArray { key: ak, value: av }
                        | Atomic::TNonEmptyArray { key: ak, value: av } => {
                            infer_from_pair(pk, ak, template_names, bindings);
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        _ => {}
                    }
                }
            }

            // list<T> matched against list<t_ty>
            Atomic::TList { value: pv } | Atomic::TNonEmptyList { value: pv } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TList { value: av } | Atomic::TNonEmptyList { value: av } => {
                            infer_from_pair(pv, av, template_names, bindings);
                        }
                        _ => {}
                    }
                }
            }

            // ClassName<T> matched against ClassName<t_ty> — or, if the bare
            // name is itself a declared template, bind it to arg_ty.
            Atomic::TNamedObject {
                fqcn: pfqcn,
                type_params: pp,
            } => {
                if pp.is_empty() && !pfqcn.contains('\\') && template_names.contains(pfqcn.as_ref())
                {
                    let bind = template_residual.as_ref().unwrap_or(arg_ty);
                    let entry = bindings.entry(pfqcn.clone()).or_insert_with(Union::empty);
                    *entry = Union::merge(entry, bind);
                    continue;
                }
                for a_atomic in &arg_ty.types {
                    if let Atomic::TNamedObject {
                        fqcn: afqcn,
                        type_params: ap,
                    } = a_atomic
                    {
                        if pfqcn == afqcn {
                            for (p_param, a_param) in pp.iter().zip(ap.iter()) {
                                infer_from_pair(p_param, a_param, template_names, bindings);
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }
}
