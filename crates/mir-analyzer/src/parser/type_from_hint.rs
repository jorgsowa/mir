use mir_types::{Atomic, Type};
/// Convert an AST `TypeHint` node into a `mir_types::Type`.
use php_ast::ast::{BuiltinType, TypeHint, TypeHintKind};

use super::name_to_string;

/// Convert a PHP AST type hint to a mir Type type.
/// `context_fqcn` is the class where this type hint appears (used for `self`/`parent`).
pub fn type_from_hint(hint: &TypeHint<'_, '_>, context_fqcn: Option<&str>) -> Type {
    match &hint.kind {
        TypeHintKind::Nullable(inner) => {
            let mut u = type_from_hint(inner, context_fqcn);
            u.add_type(Atomic::TNull);
            u
        }
        TypeHintKind::Union(parts) => {
            let mut u = Type::empty();
            for part in parts.iter() {
                for atomic in type_from_hint(part, context_fqcn).types {
                    u.add_type(atomic);
                }
            }
            u
        }
        TypeHintKind::Intersection(parts) => {
            let resolved: Vec<Type> = parts
                .iter()
                .map(|p| type_from_hint(p, context_fqcn))
                .collect();
            if resolved.is_empty() {
                Type::mixed()
            } else {
                Type::single(Atomic::TIntersection {
                    parts: mir_types::union::vec_to_type_params(resolved),
                })
            }
        }
        TypeHintKind::Keyword(builtin, _span) => builtin_type_to_union(*builtin, context_fqcn),
        TypeHintKind::Named(name) => {
            let name_str = name_to_string(name);
            named_type_to_union(&name_str, context_fqcn)
        }
    }
}

fn builtin_type_to_union(ty: BuiltinType, context_fqcn: Option<&str>) -> Type {
    match ty {
        BuiltinType::Int | BuiltinType::Integer => Type::single(Atomic::TInt),
        BuiltinType::Float | BuiltinType::Double => Type::single(Atomic::TFloat),
        BuiltinType::String => Type::single(Atomic::TString),
        BuiltinType::Bool | BuiltinType::Boolean => Type::single(Atomic::TBool),
        BuiltinType::Void => Type::single(Atomic::TVoid),
        BuiltinType::Never => Type::single(Atomic::TNever),
        BuiltinType::Mixed => Type::mixed(),
        BuiltinType::Object => Type::single(Atomic::TObject),
        BuiltinType::Array => Type::single(Atomic::TArray {
            key: Box::new(Type::single(Atomic::TMixed)),
            value: Box::new(Type::mixed()),
        }),
        BuiltinType::Callable => Type::single(Atomic::TCallable {
            params: None,
            return_type: None,
        }),
        BuiltinType::Iterable => Type::single(Atomic::TArray {
            key: Box::new(Type::single(Atomic::TMixed)),
            value: Box::new(Type::mixed()),
        }),
        BuiltinType::Null => Type::single(Atomic::TNull),
        BuiltinType::True => Type::single(Atomic::TTrue),
        BuiltinType::False => Type::single(Atomic::TFalse),
        BuiltinType::Self_ => {
            if let Some(fqcn) = context_fqcn {
                Type::single(Atomic::TSelf { fqcn: fqcn.into() })
            } else {
                Type::single(Atomic::TObject)
            }
        }
        BuiltinType::Parent_ => {
            if let Some(fqcn) = context_fqcn {
                Type::single(Atomic::TParent { fqcn: fqcn.into() })
            } else {
                Type::single(Atomic::TObject)
            }
        }
        BuiltinType::Static => {
            if let Some(fqcn) = context_fqcn {
                Type::single(Atomic::TStaticObject { fqcn: fqcn.into() })
            } else {
                Type::single(Atomic::TObject)
            }
        }
    }
}

fn named_type_to_union(name: &str, context_fqcn: Option<&str>) -> Type {
    match name.to_lowercase().as_str() {
        "self" => {
            if let Some(fqcn) = context_fqcn {
                Type::single(Atomic::TSelf { fqcn: fqcn.into() })
            } else {
                Type::single(Atomic::TObject)
            }
        }
        "parent" => {
            if let Some(fqcn) = context_fqcn {
                Type::single(Atomic::TParent { fqcn: fqcn.into() })
            } else {
                Type::single(Atomic::TObject)
            }
        }
        "static" => {
            if let Some(fqcn) = context_fqcn {
                Type::single(Atomic::TStaticObject { fqcn: fqcn.into() })
            } else {
                Type::single(Atomic::TObject)
            }
        }
        _ => Type::single(Atomic::TNamedObject {
            fqcn: normalize_fqcn(name).into(),
            type_params: mir_types::union::empty_type_params(),
        }),
    }
}

fn normalize_fqcn(s: &str) -> String {
    s.trim_start_matches('\\').to_string()
}

/// Same as [`type_from_hint`] but for the owned (lifetime-free) AST.
pub fn type_from_hint_owned(hint: &php_ast::owned::TypeHint, context_fqcn: Option<&str>) -> Type {
    match &hint.kind {
        php_ast::owned::TypeHintKind::Nullable(inner) => {
            let mut u = type_from_hint_owned(inner, context_fqcn);
            u.add_type(Atomic::TNull);
            u
        }
        php_ast::owned::TypeHintKind::Union(parts) => {
            let mut u = Type::empty();
            for part in parts.iter() {
                for atomic in type_from_hint_owned(part, context_fqcn).types {
                    u.add_type(atomic);
                }
            }
            u
        }
        php_ast::owned::TypeHintKind::Intersection(parts) => {
            let resolved: Vec<Type> = parts
                .iter()
                .map(|p| type_from_hint_owned(p, context_fqcn))
                .collect();
            if resolved.is_empty() {
                Type::mixed()
            } else {
                Type::single(Atomic::TIntersection {
                    parts: mir_types::union::vec_to_type_params(resolved),
                })
            }
        }
        php_ast::owned::TypeHintKind::Keyword(builtin, _span) => {
            builtin_type_to_union(*builtin, context_fqcn)
        }
        php_ast::owned::TypeHintKind::Named(name) => {
            let name_str = super::name_to_string_owned(name);
            named_type_to_union(&name_str, context_fqcn)
        }
    }
}
