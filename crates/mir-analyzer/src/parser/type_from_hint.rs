/// Convert an AST `TypeHint` node into a `mir_types::Union`.
use php_ast::ast::{BuiltinType, TypeHint, TypeHintKind};
use mir_types::{Atomic, Union};

use super::name_to_string;

/// Convert a PHP AST type hint to a mir Union type.
/// `context_fqcn` is the class where this type hint appears (used for `self`/`parent`).
pub fn type_from_hint(
    hint: &TypeHint<'_, '_>,
    context_fqcn: Option<&str>,
) -> Union {
    match &hint.kind {
        TypeHintKind::Nullable(inner) => {
            let mut u = type_from_hint(inner, context_fqcn);
            u.add_type(Atomic::TNull);
            u
        }
        TypeHintKind::Union(parts) => {
            let mut u = Union::empty();
            for part in parts.iter() {
                for atomic in type_from_hint(part, context_fqcn).types {
                    u.add_type(atomic);
                }
            }
            u
        }
        TypeHintKind::Intersection(parts) => {
            // Simplification: use first part for now.
            if let Some(first) = parts.first() {
                type_from_hint(first, context_fqcn)
            } else {
                Union::mixed()
            }
        }
        TypeHintKind::Keyword(builtin, _span) => builtin_type_to_union(*builtin, context_fqcn),
        TypeHintKind::Named(name) => {
            let name_str = name_to_string(name);
            named_type_to_union(&name_str, context_fqcn)
        }
    }
}

fn builtin_type_to_union(ty: BuiltinType, context_fqcn: Option<&str>) -> Union {
    match ty {
        BuiltinType::Int | BuiltinType::Integer => Union::single(Atomic::TInt),
        BuiltinType::Float | BuiltinType::Double => Union::single(Atomic::TFloat),
        BuiltinType::String => Union::single(Atomic::TString),
        BuiltinType::Bool | BuiltinType::Boolean => Union::single(Atomic::TBool),
        BuiltinType::Void => Union::single(Atomic::TVoid),
        BuiltinType::Never => Union::single(Atomic::TNever),
        BuiltinType::Mixed => Union::mixed(),
        BuiltinType::Object => Union::single(Atomic::TObject),
        BuiltinType::Array => Union::single(Atomic::TArray {
            key: Box::new(Union::single(Atomic::TMixed)),
            value: Box::new(Union::mixed()),
        }),
        BuiltinType::Callable => Union::single(Atomic::TCallable {
            params: None,
            return_type: None,
        }),
        BuiltinType::Iterable => Union::single(Atomic::TArray {
            key: Box::new(Union::single(Atomic::TMixed)),
            value: Box::new(Union::mixed()),
        }),
        BuiltinType::Null => Union::single(Atomic::TNull),
        BuiltinType::True => Union::single(Atomic::TTrue),
        BuiltinType::False => Union::single(Atomic::TFalse),
        BuiltinType::Self_ => {
            if let Some(fqcn) = context_fqcn {
                Union::single(Atomic::TSelf { fqcn: fqcn.into() })
            } else {
                Union::single(Atomic::TObject)
            }
        }
        BuiltinType::Parent_ => {
            if let Some(fqcn) = context_fqcn {
                Union::single(Atomic::TParent { fqcn: fqcn.into() })
            } else {
                Union::single(Atomic::TObject)
            }
        }
        BuiltinType::Static => {
            if let Some(fqcn) = context_fqcn {
                Union::single(Atomic::TStaticObject { fqcn: fqcn.into() })
            } else {
                Union::single(Atomic::TObject)
            }
        }
    }
}

fn named_type_to_union(name: &str, context_fqcn: Option<&str>) -> Union {
    match name.to_lowercase().as_str() {
        "self" => {
            if let Some(fqcn) = context_fqcn {
                Union::single(Atomic::TSelf { fqcn: fqcn.into() })
            } else {
                Union::single(Atomic::TObject)
            }
        }
        "parent" => {
            if let Some(fqcn) = context_fqcn {
                Union::single(Atomic::TParent { fqcn: fqcn.into() })
            } else {
                Union::single(Atomic::TObject)
            }
        }
        "static" => {
            if let Some(fqcn) = context_fqcn {
                Union::single(Atomic::TStaticObject { fqcn: fqcn.into() })
            } else {
                Union::single(Atomic::TObject)
            }
        }
        _ => Union::single(Atomic::TNamedObject {
            fqcn: normalize_fqcn(name).into(),
            type_params: vec![],
        }),
    }
}

fn normalize_fqcn(s: &str) -> String {
    s.trim_start_matches('\\').to_string()
}
