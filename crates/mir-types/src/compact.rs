/// Compact representation of simple types for use in `FnParam` inside `TCallable`/`TClosure`.
///
/// Most parameters in vendor code are simple scalar types (string, int, bool, mixed, etc.).
/// Instead of storing full `Union` structs (176 bytes), we use this enum where:
/// - Simple scalars are stored inline (1 byte discriminant)
/// - Complex types are boxed (pointer to Union)
///
/// This reduces the size of `mir_types::atomic::FnParam::ty` from ~176 bytes to ~8-16 bytes.
use crate::atomic::Atomic;
use crate::union::Union;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SimpleType {
    String,
    Int,
    Float,
    Bool,
    Mixed,
    Null,
    Void,
    Never,
    /// Complex types (multi-union, type args, etc.) are boxed.
    Complex(Box<Union>),
}

impl SimpleType {
    /// Convert a Union into a SimpleType, boxing complex types.
    pub fn from_union(u: Union) -> Self {
        // Simple scalar: single atomic, no flags.
        if !u.possibly_undefined && !u.from_docblock && u.types.len() == 1 {
            match &u.types[0] {
                Atomic::TString => return Self::String,
                Atomic::TInt => return Self::Int,
                Atomic::TFloat => return Self::Float,
                Atomic::TBool => return Self::Bool,
                Atomic::TMixed => return Self::Mixed,
                Atomic::TNull => return Self::Null,
                Atomic::TVoid => return Self::Void,
                Atomic::TNever => return Self::Never,
                _ => {}
            }
        }
        Self::Complex(Box::new(u))
    }

    /// Convert back to a Union.
    pub fn to_union(&self) -> Union {
        match self {
            Self::String => Union::string(),
            Self::Int => Union::int(),
            Self::Float => Union::float(),
            Self::Bool => Union::bool(),
            Self::Mixed => Union::mixed(),
            Self::Null => Union::null(),
            Self::Void => Union::void(),
            Self::Never => Union::never(),
            Self::Complex(u) => *u.clone(),
        }
    }

    /// Check if this is a simple scalar (not Complex).
    pub fn is_simple(&self) -> bool {
        !matches!(self, Self::Complex(_))
    }

    /// Get as a Union reference if Complex, or None if simple.
    pub fn as_complex(&self) -> Option<&Union> {
        match self {
            Self::Complex(u) => Some(u),
            _ => None,
        }
    }
}

impl fmt::Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::Mixed => write!(f, "mixed"),
            Self::Null => write!(f, "null"),
            Self::Void => write!(f, "void"),
            Self::Never => write!(f, "never"),
            Self::Complex(u) => write!(f, "{}", u),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_scalar_roundtrip() {
        let u = Union::string();
        let s = SimpleType::from_union(u.clone());
        assert_eq!(s, SimpleType::String);
        assert_eq!(s.to_union(), u);
    }

    #[test]
    fn nullable_scalar_is_complex() {
        let u = Union::nullable(Atomic::TString);
        let s = SimpleType::from_union(u.clone());
        assert_eq!(s, SimpleType::Complex(Box::new(u.clone())));
        assert_eq!(s.to_union(), u);
    }
}
