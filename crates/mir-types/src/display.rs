use std::fmt;

use crate::atomic::Atomic;
use crate::union::Union;

impl fmt::Display for Union {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.types.is_empty() {
            return write!(f, "never");
        }
        let strs: Vec<String> = self.types.iter().map(|a| format!("{}", a)).collect();
        write!(f, "{}", strs.join("|"))
    }
}

impl fmt::Display for Atomic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Atomic::TString => write!(f, "string"),
            Atomic::TLiteralString(s) => write!(f, "\"{}\"", s),
            Atomic::TClassString(None) => write!(f, "class-string"),
            Atomic::TClassString(Some(cls)) => write!(f, "class-string<{}>", cls),
            Atomic::TNonEmptyString => write!(f, "non-empty-string"),
            Atomic::TNumericString => write!(f, "numeric-string"),

            Atomic::TInt => write!(f, "int"),
            Atomic::TLiteralInt(n) => write!(f, "{}", n),
            Atomic::TIntRange { min, max } => {
                let lo = min
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "min".to_string());
                let hi = max
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "max".to_string());
                write!(f, "int<{}, {}>", lo, hi)
            }
            Atomic::TPositiveInt => write!(f, "positive-int"),
            Atomic::TNegativeInt => write!(f, "negative-int"),
            Atomic::TNonNegativeInt => write!(f, "non-negative-int"),

            Atomic::TFloat => write!(f, "float"),
            Atomic::TLiteralFloat(i, frac) => write!(f, "{}.{}", i, frac),

            Atomic::TBool => write!(f, "bool"),
            Atomic::TTrue => write!(f, "true"),
            Atomic::TFalse => write!(f, "false"),

            Atomic::TNull => write!(f, "null"),
            Atomic::TVoid => write!(f, "void"),
            Atomic::TNever => write!(f, "never"),
            Atomic::TMixed => write!(f, "mixed"),
            Atomic::TScalar => write!(f, "scalar"),
            Atomic::TNumeric => write!(f, "numeric"),

            Atomic::TObject => write!(f, "object"),
            Atomic::TNamedObject { fqcn, type_params } => {
                if type_params.is_empty() {
                    write!(f, "{}", fqcn)
                } else {
                    let params: Vec<String> =
                        type_params.iter().map(|p| format!("{}", p)).collect();
                    write!(f, "{}<{}>", fqcn, params.join(", "))
                }
            }
            Atomic::TStaticObject { fqcn } => write!(f, "static({})", fqcn),
            Atomic::TSelf { fqcn } => write!(f, "self({})", fqcn),
            Atomic::TParent { fqcn } => write!(f, "parent({})", fqcn),

            Atomic::TCallable {
                params: None,
                return_type: None,
            } => write!(f, "callable"),
            Atomic::TCallable {
                params: Some(params),
                return_type,
            } => {
                let ps: Vec<String> = params
                    .iter()
                    .map(|p| {
                        if let Some(ty) = &p.ty {
                            format!("{}", ty)
                        } else {
                            "mixed".to_string()
                        }
                    })
                    .collect();
                let ret = return_type
                    .as_ref()
                    .map(|r| format!("{}", r))
                    .unwrap_or_else(|| "mixed".to_string());
                write!(f, "callable({}): {}", ps.join(", "), ret)
            }
            Atomic::TCallable {
                params: None,
                return_type: Some(ret),
            } => {
                write!(f, "callable(): {}", ret)
            }
            Atomic::TClosure {
                params,
                return_type,
                ..
            } => {
                let ps: Vec<String> = params
                    .iter()
                    .map(|p| {
                        if let Some(ty) = &p.ty {
                            format!("{}", ty)
                        } else {
                            "mixed".to_string()
                        }
                    })
                    .collect();
                write!(f, "Closure({}): {}", ps.join(", "), return_type)
            }

            Atomic::TArray { key, value } => {
                write!(f, "array<{}, {}>", key, value)
            }
            Atomic::TList { value } => write!(f, "list<{}>", value),
            Atomic::TNonEmptyArray { key, value } => {
                write!(f, "non-empty-array<{}, {}>", key, value)
            }
            Atomic::TNonEmptyList { value } => write!(f, "non-empty-list<{}>", value),
            Atomic::TKeyedArray { properties, .. } => {
                let entries: Vec<String> = properties
                    .iter()
                    .map(|(k, v)| {
                        let key_str = match k {
                            crate::atomic::ArrayKey::String(s) => format!("'{}'", s),
                            crate::atomic::ArrayKey::Int(n) => n.to_string(),
                        };
                        let opt = if v.optional { "?" } else { "" };
                        format!("{}{}: {}", key_str, opt, v.ty)
                    })
                    .collect();
                write!(f, "array{{{}}}", entries.join(", "))
            }

            Atomic::TTemplateParam { name, .. } => write!(f, "{}", name),
            Atomic::TConditional {
                subject,
                if_true,
                if_false,
            } => {
                write!(f, "({} is ? {} : {})", subject, if_true, if_false)
            }

            Atomic::TInterfaceString => write!(f, "interface-string"),
            Atomic::TEnumString => write!(f, "enum-string"),
            Atomic::TTraitString => write!(f, "trait-string"),
        }
    }
}
