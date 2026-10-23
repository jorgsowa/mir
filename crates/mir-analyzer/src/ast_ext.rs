use php_ast::owned::{Arg, ArgKind, ArrayElement, ArrayElementKind};

pub(crate) trait ArgExt {
    fn unpack(&self) -> bool;
    fn by_ref(&self) -> bool;
}

impl ArgExt for Arg {
    fn unpack(&self) -> bool {
        matches!(self.kind, ArgKind::Spread(_) | ArgKind::SpreadPlaceholder)
    }

    fn by_ref(&self) -> bool {
        matches!(self.kind, ArgKind::ByRef(_))
    }
}

pub(crate) trait ArrayElementExt {
    fn unpack(&self) -> bool;
    fn by_ref(&self) -> bool;
}

impl ArrayElementExt for ArrayElement {
    fn unpack(&self) -> bool {
        matches!(self.kind, ArrayElementKind::Spread(_))
    }

    fn by_ref(&self) -> bool {
        matches!(self.kind, ArrayElementKind::ByRef(_))
    }
}

pub(crate) trait ByteStringExt {
    fn as_str(&self) -> &str;
    /// Distinct non-UTF-8 values stay distinct (each byte maps to a private-use char).
    fn to_text(&self) -> std::borrow::Cow<'_, str>;
}

impl ByteStringExt for php_ast::owned::ByteString {
    fn as_str(&self) -> &str {
        self.to_str().unwrap_or("\u{fffd}")
    }

    fn to_text(&self) -> std::borrow::Cow<'_, str> {
        match self.to_str() {
            Some(s) => std::borrow::Cow::Borrowed(s),
            None => std::borrow::Cow::Owned(
                self.as_bytes()
                    .iter()
                    .filter_map(|&b| char::from_u32(0xF700 + u32::from(b)))
                    .collect(),
            ),
        }
    }
}

pub(crate) trait NameRefExt {
    /// The text of a function, class or constant reference as the parser wrote it
    /// (`\Foo\bar` keeps its leading backslash).
    fn name_ref(&self) -> Option<std::borrow::Cow<'_, str>>;
    /// Whether `name_ref()` would be `Some`, without building the text.
    fn is_name_ref(&self) -> bool;
}

impl NameRefExt for php_ast::owned::ExprKind {
    fn name_ref(&self) -> Option<std::borrow::Cow<'_, str>> {
        use php_ast::ast::NameKind;
        use php_ast::owned::ExprKind;
        match self {
            ExprKind::Name(n) => Some(match (n.kind, &*n.parts) {
                (NameKind::Unqualified, [only]) => std::borrow::Cow::Borrowed(only),
                (NameKind::Error, _) => return None,
                (NameKind::Relative, _) => {
                    std::borrow::Cow::Owned(format!("namespace\\{}", n.join_parts()))
                }
                _ => std::borrow::Cow::Owned(n.to_string_repr()),
            }),
            _ => None,
        }
    }

    fn is_name_ref(&self) -> bool {
        use php_ast::ast::NameKind;
        use php_ast::owned::ExprKind;
        matches!(self, ExprKind::Name(n) if n.kind != NameKind::Error)
    }
}

pub(crate) trait NameBareExt {
    /// A single-segment reference without its leading backslash; `None` for
    /// namespaced ones.
    fn name_bare(&self) -> Option<&str>;
}

impl NameBareExt for php_ast::owned::ExprKind {
    fn name_bare(&self) -> Option<&str> {
        use php_ast::ast::NameKind;
        use php_ast::owned::ExprKind;
        match self {
            ExprKind::Name(n) => match (n.kind, &*n.parts) {
                (NameKind::Unqualified | NameKind::FullyQualified, [only]) => Some(only),
                _ => None,
            },
            _ => None,
        }
    }
}

pub(crate) trait NameResolveExt {
    /// The fully qualified name a function, class or constant reference resolves to.
    fn resolve_name_ref(&self, db: &dyn crate::db::MirDatabase, file: &str) -> Option<String>;
}

impl NameResolveExt for php_ast::owned::ExprKind {
    fn resolve_name_ref(&self, db: &dyn crate::db::MirDatabase, file: &str) -> Option<String> {
        use php_ast::ast::NameKind;
        let php_ast::owned::ExprKind::Name(n) = self else {
            return self
                .name_ref()
                .map(|name| crate::db::resolve_name(db, file, &name));
        };
        Some(match (n.kind, &*n.parts) {
            (NameKind::Error, _) => return None,
            (NameKind::FullyQualified, _) => n.join_parts(),
            (NameKind::Relative, _) => crate::db::prefix_file_namespace(db, file, &n.join_parts()),
            (_, [only]) => crate::db::resolve_name(db, file, only),
            _ => crate::db::resolve_name(db, file, &n.join_parts()),
        })
    }
}
