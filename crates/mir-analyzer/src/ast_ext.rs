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

/// A function reference resolved the way PHP resolves a call.
pub(crate) struct FunctionRef<'a> {
    /// The namespaced, imported or fully qualified name, tried first.
    pub qualified: std::borrow::Cow<'a, str>,
    /// The global name an unqualified call in a namespace falls back to.
    pub global: Option<&'a str>,
}

impl<'a> FunctionRef<'a> {
    /// A string callable, which never goes through the namespace or imports.
    pub fn from_text(text: &'a str) -> Self {
        Self {
            qualified: std::borrow::Cow::Borrowed(text.strip_prefix('\\').unwrap_or(text)),
            global: None,
        }
    }

    pub fn find(
        &self,
        db: &dyn crate::db::MirDatabase,
        file: &str,
    ) -> Option<(&str, std::sync::Arc<mir_codebase::definitions::FunctionDef>)> {
        let find = |name: &str| {
            crate::db::find_function_from(db, file, crate::db::Fqcn::from_str(db, name))
        };
        if let Some(f) = find(&self.qualified) {
            return Some((&self.qualified, f));
        }
        let global = self.global?;
        find(global).map(|f| (global, f))
    }

    /// The name the call resolves to; the qualified one when neither is declared.
    pub fn resolve(&self, db: &dyn crate::db::MirDatabase, file: &str) -> &str {
        self.find(db, file)
            .map_or(&self.qualified, |(name, _)| name)
    }

    /// The global function this call names, unless a namespaced function shadows it.
    pub fn global_name(&self, db: &dyn crate::db::MirDatabase, file: &str) -> Option<&str> {
        match self.global {
            Some(global) => crate::db::find_function_from(
                db,
                file,
                crate::db::Fqcn::from_str(db, &self.qualified),
            )
            .is_none()
            .then_some(global),
            None => (!self.qualified.contains('\\')).then_some(&*self.qualified),
        }
    }
}

pub(crate) trait FunctionRefExt {
    fn function_ref(&self, db: &dyn crate::db::MirDatabase, file: &str) -> Option<FunctionRef<'_>>;
}

impl FunctionRefExt for php_ast::owned::ExprKind {
    fn function_ref(&self, db: &dyn crate::db::MirDatabase, file: &str) -> Option<FunctionRef<'_>> {
        use php_ast::ast::NameKind;
        use std::borrow::Cow;
        let php_ast::owned::ExprKind::Name(n) = self else {
            return None;
        };
        let (qualified, global) = match (n.kind, &*n.parts) {
            (NameKind::Error, _) => return None,
            (NameKind::FullyQualified, [only]) => (Cow::Borrowed(&**only), None),
            (NameKind::FullyQualified, _) => (Cow::Owned(n.join_parts()), None),
            (NameKind::Relative, _) => (
                Cow::Owned(crate::db::prefix_file_namespace(db, file, &n.join_parts())),
                None,
            ),
            (NameKind::Unqualified, [only]) => {
                match db.file_imports(file).get(&mir_types::Name::new(only)) {
                    Some(imported) => (Cow::Owned(imported.as_str().to_string()), None),
                    None => match db.file_namespace(file) {
                        Some(ns) => (Cow::Owned(format!("{ns}\\{only}")), Some(&**only)),
                        None => (Cow::Borrowed(&**only), None),
                    },
                }
            }
            _ => (
                Cow::Owned(crate::db::resolve_name(db, file, &n.join_parts())),
                None,
            ),
        };
        Some(FunctionRef { qualified, global })
    }
}
