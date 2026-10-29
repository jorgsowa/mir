use super::StatementsAnalyzer;
use mir_issues::IssueKind;
use mir_types::{Atomic, Type};
use php_ast::ast::NameKind;
use php_ast::owned::{
    ArrayElementKind, ClassConstDecl, ClassMember, ClassMemberKind, Expr, ExprKind, PropertyDecl,
    TypeHint,
};

/// Nesting limit for same-class constants that refer to each other.
const MAX_CONST_DEPTH: u8 = 8;

/// A constant-expression value its declared type rejects.
struct RejectedValue {
    /// PHP's name for the value's type (`string`, `null`, a class name).
    value: String,
    declared: Type,
    /// The declared type as PHP prints it.
    type_text: String,
    /// Whether PHP rejects it while compiling the class rather than on first use.
    at_compile_time: bool,
}

impl<'a> StatementsAnalyzer<'a> {
    /// A typed property default built from constants whose value the type rejects; literal
    /// defaults are the parser's `default-value-type`.
    pub(crate) fn property_default_type_issue(
        &self,
        fqcn: &str,
        prop: &PropertyDecl,
        members: &[ClassMember],
        default: &Expr,
        default_ty: &Type,
    ) -> Option<IssueKind> {
        let name = prop.name.as_str()?;
        let rejected =
            self.rejected_value(prop.type_hint.as_ref()?, fqcn, members, default, default_ty)?;
        if !rejected.at_compile_time {
            return Some(IssueKind::InvalidPropertyAssignment {
                property: name.to_string(),
                expected: rejected.declared.to_string(),
                actual: rejected.value,
            });
        }
        let RejectedValue {
            mut value,
            type_text,
            ..
        } = rejected;
        if self.php_version >= crate::php_version::PhpVersion::new(8, 3) {
            if let Some(b) = self.folded_bool(default, fqcn, members, 0) {
                value = b.to_string();
            }
        }
        let message = if value == "null" {
            let nullable = if rejected.declared.types.len() == 1 {
                format!("?{type_text}")
            } else {
                format!("{type_text}|null")
            };
            format!(
                "Default value for property of type {type_text} may not be null. Use the nullable type {nullable} to allow null default value"
            )
        } else {
            format!(
                "Cannot use {value} as default value for property {fqcn}::${name} of type {type_text}"
            )
        };
        Some(IssueKind::CompileError {
            message,
            code: "default-value-type".to_string(),
        })
    }

    /// A typed class constant built from constants whose value PHP rejects while compiling
    /// the class; literal values are the parser's `class-constant-value-type`.
    pub(crate) fn class_const_value_type_issue(
        &self,
        fqcn: &str,
        constant: &ClassConstDecl,
        members: &[ClassMember],
        value_ty: &Type,
    ) -> Option<IssueKind> {
        let name = constant.name.as_str()?;
        let hint = constant.type_hint.as_deref()?;
        let rejected = self.rejected_value(hint, fqcn, members, &constant.value, value_ty)?;
        rejected.at_compile_time.then(|| {
            let RejectedValue { value, type_text, .. } = rejected;
            IssueKind::CompileError {
                message: format!(
                    "Cannot use {value} as value for class constant {fqcn}::{name} of type {type_text}"
                ),
                code: "class-constant-value-type".to_string(),
            }
        })
    }

    fn rejected_value(
        &self,
        hint: &TypeHint,
        fqcn: &str,
        members: &[ClassMember],
        expr: &Expr,
        ty: &Type,
    ) -> Option<RejectedValue> {
        if !refers_to_constant(expr) || refers_to_static(expr) {
            return None;
        }
        let value = php_value_type(ty)?;
        let declared = crate::expr::helpers::resolve_named_objects_in_union_native(
            crate::parser::type_from_hint_owned(hint, Some(fqcn)),
            self.db,
            &self.file,
        );
        if accepts(&declared, ty) {
            return None;
        }
        let is_enum_case = ty.types.iter().any(|a| a.is_object());
        Some(RejectedValue {
            value,
            type_text: crate::util::php_type_text(&declared, self.php_version),
            declared,
            at_compile_time: !is_enum_case && self.folds_at_compile_time(expr, fqcn, members, 0),
        })
    }

    /// The boolean `expr` folds to from literals, `!` and the class's own earlier constants.
    fn folded_bool(
        &self,
        expr: &Expr,
        fqcn: &str,
        members: &[ClassMember],
        depth: u8,
    ) -> Option<bool> {
        match &expr.kind {
            ExprKind::Bool(b) => Some(*b),
            ExprKind::Parenthesized(inner) => self.folded_bool(inner, fqcn, members, depth),
            ExprKind::UnaryPrefix(u) if u.op == php_ast::ast::UnaryPrefixOp::BooleanNot => self
                .folded_bool(&u.operand, fqcn, members, depth)
                .map(|b| !b),
            ExprKind::ClassConstAccess(access) if depth < MAX_CONST_DEPTH => {
                let (ExprKind::Name(class), ExprKind::Identifier(member)) =
                    (&access.class.kind, &access.member.kind)
                else {
                    return None;
                };
                let class = crate::ast_ext::resolve_ast_name(class, self.db, &self.file);
                if class != "self" && !class.eq_ignore_ascii_case(fqcn) {
                    return None;
                }
                let value = own_constant_before(members, member, expr.span.start)?;
                self.folded_bool(value, fqcn, members, depth + 1)
            }
            _ => None,
        }
    }

    /// Whether PHP evaluates `expr` while compiling the class: literals, builtin constants
    /// written so no namespace fallback applies, and the class's own constants.
    fn folds_at_compile_time(
        &self,
        expr: &Expr,
        fqcn: &str,
        members: &[ClassMember],
        depth: u8,
    ) -> bool {
        let folds = |e: &Expr| self.folds_at_compile_time(e, fqcn, members, depth);
        match &expr.kind {
            ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::String(_)
            | ExprKind::Nowdoc { .. }
            | ExprKind::Bool(_)
            | ExprKind::Null
            | ExprKind::MagicConst(_) => true,
            ExprKind::Parenthesized(inner) => folds(inner),
            ExprKind::UnaryPrefix(u) => folds(&u.operand),
            ExprKind::Binary(b) => folds(&b.left) && folds(&b.right),
            ExprKind::NullCoalesce(n) => folds(&n.left) && folds(&n.right),
            ExprKind::Ternary(t) => {
                folds(&t.condition)
                    && t.then_expr.as_deref().is_none_or(folds)
                    && folds(&t.else_expr)
            }
            ExprKind::Array(elements) => elements.iter().all(|el| {
                el.key.as_ref().is_none_or(folds)
                    && matches!(&el.kind, ArrayElementKind::Value(v) if folds(v))
            }),
            ExprKind::Name(name) => {
                let global_scope = name.kind == NameKind::FullyQualified
                    || name.kind == NameKind::Unqualified && !fqcn.contains('\\');
                global_scope && crate::stubs::is_builtin_constant(&name.join_parts())
            }
            ExprKind::ClassConstAccess(access) => {
                let ExprKind::Identifier(member) = &access.member.kind else {
                    return false;
                };
                let ExprKind::Name(class) = &access.class.kind else {
                    return false;
                };
                let class = crate::ast_ext::resolve_ast_name(class, self.db, &self.file);
                if class == "static" || class == "parent" {
                    return false;
                }
                if member.eq_ignore_ascii_case("class") {
                    return true;
                }
                if class != "self" && !class.eq_ignore_ascii_case(fqcn) {
                    return crate::stubs::stub_path_for_class(&class).is_some();
                }
                depth < MAX_CONST_DEPTH
                    && own_constant_before(members, member, expr.span.start).is_some_and(|value| {
                        self.folds_at_compile_time(value, fqcn, members, depth + 1)
                    })
            }
            _ => false,
        }
    }
}

/// The value of the class's constant `name`, if it is declared before `offset`.
fn own_constant_before<'m>(
    members: &'m [ClassMember],
    name: &str,
    offset: u32,
) -> Option<&'m Expr> {
    members.iter().find_map(|member| match &member.kind {
        ClassMemberKind::ClassConst(c)
            if c.name.as_str() == Some(name) && member.span.end <= offset =>
        {
            Some(&c.value)
        }
        _ => None,
    })
}

/// Whether `expr` contains `static::`, which the parser rejects in a constant expression.
fn refers_to_static(expr: &Expr) -> bool {
    struct FindStatic(bool);
    impl php_ast::owned::visitor::OwnedVisitor for FindStatic {
        type Break = ();
        fn visit_expr(&mut self, expr: &Expr) -> std::ops::ControlFlow<()> {
            if let ExprKind::ClassConstAccess(access) = &expr.kind {
                if let ExprKind::Name(class) = &access.class.kind {
                    if class.kind == NameKind::Unqualified
                        && class.join_parts().eq_ignore_ascii_case("static")
                    {
                        self.0 = true;
                        return std::ops::ControlFlow::Break(());
                    }
                }
            }
            crate::recursion::walk_expr(self, expr)
        }
    }
    let mut finder = FindStatic(false);
    let _ = php_ast::owned::visitor::OwnedVisitor::visit_expr(&mut finder, expr);
    finder.0
}

/// Whether `expr` reads a global or class constant anywhere.
pub(crate) fn refers_to_constant(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Name(_) | ExprKind::ClassConstAccess(_) => true,
        ExprKind::Parenthesized(inner) => refers_to_constant(inner),
        ExprKind::UnaryPrefix(u) => refers_to_constant(&u.operand),
        ExprKind::Binary(b) => refers_to_constant(&b.left) || refers_to_constant(&b.right),
        ExprKind::NullCoalesce(n) => refers_to_constant(&n.left) || refers_to_constant(&n.right),
        ExprKind::Ternary(t) => {
            refers_to_constant(&t.condition)
                || t.then_expr.as_deref().is_some_and(refers_to_constant)
                || refers_to_constant(&t.else_expr)
        }
        ExprKind::Array(elements) => elements.iter().any(|el| {
            el.key.as_ref().is_some_and(refers_to_constant) || refers_to_constant(el.value())
        }),
        _ => false,
    }
}

/// PHP's name for the type of every value `ty` holds, when they all share one.
fn php_value_type(ty: &Type) -> Option<String> {
    let mut found: Option<String> = None;
    for atomic in &ty.types {
        let name = match atomic {
            Atomic::TString
            | Atomic::TLiteralString(_)
            | Atomic::TNonEmptyString
            | Atomic::TNumericString
            | Atomic::TCallableString
            | Atomic::TClassString(_)
            | Atomic::TInterfaceString(_)
            | Atomic::TEnumString
            | Atomic::TTraitString => "string",
            Atomic::TInt
            | Atomic::TLiteralInt(_)
            | Atomic::TIntRange { .. }
            | Atomic::TPositiveInt
            | Atomic::TNegativeInt
            | Atomic::TNonNegativeInt => "int",
            Atomic::TFloat | Atomic::TLiteralFloat(..) | Atomic::TIntegralFloat => "float",
            Atomic::TBool | Atomic::TTrue | Atomic::TFalse => "bool",
            Atomic::TNull => "null",
            Atomic::TArray { .. }
            | Atomic::TList { .. }
            | Atomic::TNonEmptyArray { .. }
            | Atomic::TNonEmptyList { .. }
            | Atomic::TKeyedArray { .. } => "array",
            // Properties can't default to `new`, so an object is an enum case.
            Atomic::TLiteralEnumCase {
                enum_fqcn: fqcn, ..
            }
            | Atomic::TNamedObject { fqcn, .. } => fqcn.as_ref(),
            _ => return None,
        };
        match &found {
            Some(seen) if seen != name => return None,
            Some(_) => {}
            None => found = Some(name.trim_start_matches('\\').to_string()),
        }
    }
    found
}

/// Whether a property declared `declared` accepts every value in `value` as its default; types
/// this does not model are accepted.
fn accepts(declared: &Type, value: &Type) -> bool {
    let has = |pred: fn(&Atomic) -> bool| declared.types.iter().any(pred);
    if has(|a| !is_modelled_declared(a)) {
        return true;
    }
    value.types.iter().all(|atomic| match atomic {
        Atomic::TNull => has(|a| matches!(a, Atomic::TNull)),
        Atomic::TTrue => has(|a| matches!(a, Atomic::TBool | Atomic::TTrue)),
        Atomic::TFalse => has(|a| matches!(a, Atomic::TBool | Atomic::TFalse)),
        Atomic::TBool => has(|a| matches!(a, Atomic::TBool | Atomic::TTrue | Atomic::TFalse)),
        Atomic::TFloat | Atomic::TLiteralFloat(..) | Atomic::TIntegralFloat => {
            has(|a| matches!(a, Atomic::TFloat))
        }
        Atomic::TLiteralEnumCase { .. } | Atomic::TNamedObject { .. } => has(|a| {
            matches!(
                a,
                Atomic::TObject | Atomic::TNamedObject { .. } | Atomic::TSelf { .. }
            )
        }),
        a if a.is_int() => has(|a| matches!(a, Atomic::TInt | Atomic::TFloat)),
        a if a.is_string() => has(|a| matches!(a, Atomic::TString)),
        a if a.is_array() => has(|a| matches!(a, Atomic::TArray { .. })),
        _ => true,
    })
}

fn is_modelled_declared(atomic: &Atomic) -> bool {
    matches!(
        atomic,
        Atomic::TInt
            | Atomic::TFloat
            | Atomic::TString
            | Atomic::TBool
            | Atomic::TTrue
            | Atomic::TFalse
            | Atomic::TNull
            | Atomic::TArray { .. }
            | Atomic::TObject
            | Atomic::TNamedObject { .. }
            | Atomic::TSelf { .. }
    )
}
