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
}

impl NameRefExt for php_ast::owned::ExprKind {
    fn name_ref(&self) -> Option<std::borrow::Cow<'_, str>> {
        use php_ast::ast::NameKind;
        use php_ast::owned::ExprKind;
        match self {
            ExprKind::Identifier(s) => Some(std::borrow::Cow::Borrowed(s)),
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
            ExprKind::Identifier(s) => Some(s.trim_start_matches('\\')),
            ExprKind::Name(n) => match (n.kind, &*n.parts) {
                (NameKind::Unqualified | NameKind::FullyQualified, [only]) => Some(only),
                _ => None,
            },
            _ => None,
        }
    }
}
