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

    for (param, arg_ty) in params.iter().zip(arg_types.iter()) {
        if let Some(param_ty) = &param.ty {
            infer_from_pair(param_ty, arg_ty, &mut bindings);
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

/// Recursively match `param_ty` (which may contain template placeholders)
/// against `arg_ty` (a concrete type), updating `bindings`.
fn infer_from_pair(param_ty: &Union, arg_ty: &Union, bindings: &mut HashMap<Arc<str>, Union>) {
    for p_atomic in &param_ty.types {
        match p_atomic {
            // Direct template placeholder: T → bind T = arg_ty
            Atomic::TTemplateParam { name, .. } => {
                // Merge if already partially bound
                let entry = bindings.entry(name.clone()).or_insert_with(Union::empty);
                *entry = Union::merge(entry, arg_ty);
            }

            // array<K, V> matched against array<k_ty, v_ty>
            Atomic::TArray { key: pk, value: pv } => {
                for a_atomic in &arg_ty.types {
                    match a_atomic {
                        Atomic::TArray { key: ak, value: av }
                        | Atomic::TNonEmptyArray { key: ak, value: av } => {
                            infer_from_pair(pk, ak, bindings);
                            infer_from_pair(pv, av, bindings);
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
                            infer_from_pair(pv, av, bindings);
                        }
                        _ => {}
                    }
                }
            }

            // ClassName<T> matched against ClassName<t_ty>
            Atomic::TNamedObject {
                fqcn: pfqcn,
                type_params: pp,
            } => {
                for a_atomic in &arg_ty.types {
                    if let Atomic::TNamedObject {
                        fqcn: afqcn,
                        type_params: ap,
                    } = a_atomic
                    {
                        if pfqcn == afqcn {
                            for (p_param, a_param) in pp.iter().zip(ap.iter()) {
                                infer_from_pair(p_param, a_param, bindings);
                            }
                        }
                    }
                }
            }

            _ => {}
        }
    }
}
