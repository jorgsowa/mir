//! Attribute validation — checks `#[...]` attribute usages against PHP rules.
//!
//! ## Checks performed
//!
//! ### Structural checks (AST only)
//! - `#[Attribute]` used on function / method / property / parameter → `InvalidAttribute`
//! - Class decorated with `#[Attribute]` is abstract → `InvalidAttribute`
//! - Interface decorated with `#[Attribute]` → `InvalidAttribute`
//! - Trait decorated with `#[Attribute]` → `InvalidAttribute`
//! - Attribute class has a private constructor → `InvalidAttribute`
//!
//! ### Cross-file checks (requires database)
//! - Class used as `#[SomeClass]` does not have `#[Attribute]` annotation → `InvalidAttribute`
//! - Attribute applied to element that doesn't match its declared target → `InvalidAttribute`
//! - Same non-repeatable attribute applied twice on the same element → `InvalidAttribute`

use std::sync::Arc;

use mir_issues::{Issue, IssueKind, Location};
use php_ast::owned::{
    Attribute, ClassDecl, ClassMemberKind, EnumDecl, EnumMemberKind, Expr, ExprKind, FunctionDecl,
    InterfaceDecl, PropertyHook, TraitDecl,
};
use php_rs_parser::source_map::SourceMap;

const ATTR_IS_REPEATABLE: i64 = 64;
const ATTR_TARGET_ALL: i64 = 63;
use crate::db::{find_class_like, resolve_name, Fqcn, MirDatabase};
use crate::diagnostics::offset_to_line_col;

// ---------------------------------------------------------------------------
// Target bitmask constants (mirror PHP's Attribute class)
// ---------------------------------------------------------------------------

const TARGET_CLASS: i64 = 1;
const TARGET_FUNCTION: i64 = 2;
const TARGET_METHOD: i64 = 4;
const TARGET_PROPERTY: i64 = 8;
const TARGET_CLASS_CONSTANT: i64 = 16;
const TARGET_PARAMETER: i64 = 32;

// ---------------------------------------------------------------------------
// Location helpers
// ---------------------------------------------------------------------------

fn span_to_location(
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    start: u32,
    end: u32,
) -> Location {
    let (line, col_start) = offset_to_line_col(source, start, source_map);
    let (line_end, col_end) = offset_to_line_col(source, end, source_map);
    Location {
        file: file.clone(),
        line,
        line_end,
        col_start,
        col_end,
    }
}

fn invalid_attr(message: impl Into<String>, loc: Location) -> Issue {
    Issue::new(
        IssueKind::InvalidAttribute {
            message: message.into(),
        },
        loc,
    )
}

// ---------------------------------------------------------------------------
// Class references inside attribute constructor arguments
// ---------------------------------------------------------------------------

/// Recursively walk a constant-expression tree (the only kind of expression
/// PHP allows inside an attribute argument) recording a usage reference for
/// every class name reached via `Foo::class`, `Foo::CONST`, or an enum case
/// `Foo::Case` — all three share the same `ClassConstAccess` AST shape.
/// `self`/`static`/`parent` are skipped: attribute arguments have no
/// surrounding method scope to resolve them against.
fn record_class_refs_in_expr(
    expr: &Expr,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
) {
    let record = |name: &str, span: php_ast::Span| {
        let resolved = resolve_name(db, file.as_ref(), name);
        if matches!(resolved.as_str(), "self" | "static" | "parent") {
            return;
        }
        let (line, col_start) = offset_to_line_col(source, span.start, source_map);
        let (line_end, col_end) = offset_to_line_col(source, span.end, source_map);
        db.record_reference_location(crate::db::RefLoc {
            symbol_key: Arc::from(format!("cls:{resolved}")),
            file: file.clone(),
            line,
            col_start,
            col_end: crate::diagnostics::clamp_col_end(line, line_end, col_start, col_end),
        });
    };

    match &expr.kind {
        ExprKind::ClassConstAccess(cca) => {
            if let ExprKind::Identifier(id) = &cca.class.kind {
                record(id.as_ref(), cca.class.span);
                // `Foo::CONST`/`Foo::Case` (but not `Foo::class`, which has no
                // constant/case of its own) is also a real reference to that
                // specific constant/enum case — without this, `#[Attr(Status::Active)]`
                // marked `Status` used but not `Status::Active`, so find-references
                // and dead-code analysis both missed this usage.
                if !matches!(&cca.member.kind, ExprKind::Identifier(m) if m.as_ref() == "class") {
                    if let ExprKind::Identifier(member) = &cca.member.kind {
                        let resolved = resolve_name(db, file.as_ref(), id.as_ref());
                        if !matches!(resolved.as_str(), "self" | "static" | "parent") {
                            let (line, col_start) =
                                offset_to_line_col(source, cca.member.span.start, source_map);
                            let (line_end, col_end) =
                                offset_to_line_col(source, cca.member.span.end, source_map);
                            db.record_reference_location(crate::db::RefLoc {
                                symbol_key: Arc::from(format!("cnst:{resolved}::{member}")),
                                file: file.clone(),
                                line,
                                col_start,
                                col_end: crate::diagnostics::clamp_col_end(
                                    line, line_end, col_start, col_end,
                                ),
                            });
                        }
                    }
                }
            } else {
                record_class_refs_in_expr(&cca.class, db, file, source, source_map);
            }
        }
        ExprKind::New(n) => {
            if let ExprKind::Identifier(id) = &n.class.kind {
                record(id.as_ref(), n.class.span);
            } else {
                record_class_refs_in_expr(&n.class, db, file, source, source_map);
            }
            for arg in n.args.iter() {
                record_class_refs_in_expr(&arg.value, db, file, source, source_map);
            }
        }
        ExprKind::Array(elements) => {
            for el in elements.iter() {
                if let Some(key) = &el.key {
                    record_class_refs_in_expr(key, db, file, source, source_map);
                }
                record_class_refs_in_expr(&el.value, db, file, source, source_map);
            }
        }
        ExprKind::Binary(b) => {
            record_class_refs_in_expr(&b.left, db, file, source, source_map);
            record_class_refs_in_expr(&b.right, db, file, source, source_map);
        }
        ExprKind::Ternary(t) => {
            record_class_refs_in_expr(&t.condition, db, file, source, source_map);
            if let Some(then_expr) = &t.then_expr {
                record_class_refs_in_expr(then_expr, db, file, source, source_map);
            }
            record_class_refs_in_expr(&t.else_expr, db, file, source, source_map);
        }
        ExprKind::UnaryPrefix(u) => {
            record_class_refs_in_expr(&u.operand, db, file, source, source_map);
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Attribute name helper
// ---------------------------------------------------------------------------

fn is_attribute_class_annotation(attr: &Attribute) -> bool {
    attr.name
        .parts
        .last()
        .map(|p| p.as_ref().eq_ignore_ascii_case("Attribute"))
        .unwrap_or(false)
}

/// Resolve the fully-qualified name of an attribute reference in `file` context.
fn resolve_attr_name(db: &dyn MirDatabase, file: &str, attr: &Attribute) -> String {
    let raw = attr
        .name
        .parts
        .iter()
        .map(|p| p.as_ref())
        .collect::<Vec<_>>()
        .join("\\");
    // `attr.name.parts` drops the leading backslash, so a fully-qualified
    // attribute like `#[\Override]` would otherwise be re-resolved against the
    // file namespace (→ `App\Console\Override`). Restore the leading `\` so
    // `resolve_name` treats it as the global name it is.
    if matches!(attr.name.kind, php_ast::ast::NameKind::FullyQualified) {
        return resolve_name(db, file, &format!("\\{raw}"));
    }
    resolve_name(db, file, &raw)
}

// ---------------------------------------------------------------------------
// Structural checks (no database needed)
// ---------------------------------------------------------------------------

/// Check that `#[Attribute]` (the PHP built-in) is not placed on a function or its parameters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_function_attributes(
    decl: &FunctionDecl,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    for attr in decl.attributes.iter() {
        if !is_attribute_class_annotation(attr) {
            continue;
        }
        let loc = span_to_location(file, source, source_map, attr.span.start, attr.span.end);
        issues.push(invalid_attr(
            "#[Attribute] can only be applied to classes, not functions",
            loc,
        ));
    }
    check_attribute_list(
        &decl.attributes,
        TARGET_FUNCTION,
        db,
        file,
        source,
        source_map,
        issues,
        record_refs,
        all_symbols.as_deref_mut(),
    );
    for param in decl.params.iter() {
        // `#[Attribute]` on a function parameter is invalid
        for attr in param.attributes.iter() {
            if is_attribute_class_annotation(attr) {
                let loc =
                    span_to_location(file, source, source_map, attr.span.start, attr.span.end);
                issues.push(invalid_attr(
                    "#[Attribute] can only be applied to classes, not parameters",
                    loc,
                ));
            }
        }
        check_attribute_list(
            &param.attributes,
            TARGET_PARAMETER,
            db,
            file,
            source,
            source_map,
            issues,
            record_refs,
            all_symbols.as_deref_mut(),
        );
    }
}

/// Check attribute placement rules for a class declaration.
///
/// - `#[Attribute]` on abstract class → invalid
/// - `#[Attribute]` class with private constructor → invalid
/// - All `#[...]` attributes: validate against database if possible
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_class_attributes(
    decl: &ClassDecl,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    // Check 1: `#[Attribute]` on abstract class
    if decl.modifiers.is_abstract {
        for attr in decl.attributes.iter() {
            if !is_attribute_class_annotation(attr) {
                continue;
            }
            let loc = span_to_location(file, source, source_map, attr.span.start, attr.span.end);
            issues.push(invalid_attr(
                "Abstract classes cannot be attribute classes",
                loc,
            ));
        }
    }

    // Check 2: `#[Attribute]` class has private constructor
    let class_has_attribute = decl.attributes.iter().any(is_attribute_class_annotation);
    if class_has_attribute {
        for member in decl.body.members.iter() {
            let ClassMemberKind::Method(method) = &member.kind else {
                continue;
            };
            let method_name = method.name.as_deref().unwrap_or("");
            if !method_name.eq_ignore_ascii_case("__construct") {
                continue;
            }
            if matches!(method.visibility, Some(php_ast::ast::Visibility::Private)) {
                let loc =
                    span_to_location(file, source, source_map, member.span.start, member.span.end);
                issues.push(invalid_attr(
                    "Attribute class constructor must not be private",
                    loc,
                ));
            }
        }
    }

    // Check 3: Validate `#[...]` attribute usages against the database for
    // the class itself, its methods, and their parameters.
    check_attribute_list(
        &decl.attributes,
        TARGET_CLASS,
        db,
        file,
        source,
        source_map,
        issues,
        record_refs,
        all_symbols.as_deref_mut(),
    );

    for member in decl.body.members.iter() {
        match &member.kind {
            ClassMemberKind::Method(method) => {
                check_attribute_list(
                    &method.attributes,
                    TARGET_METHOD,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                for param in method.params.iter() {
                    check_attribute_list(
                        &param.attributes,
                        TARGET_PARAMETER,
                        db,
                        file,
                        source,
                        source_map,
                        issues,
                        record_refs,
                        all_symbols.as_deref_mut(),
                    );
                    // `#[Attribute]` on a method parameter is invalid
                    for attr in param.attributes.iter() {
                        if is_attribute_class_annotation(attr) {
                            let loc = span_to_location(
                                file,
                                source,
                                source_map,
                                attr.span.start,
                                attr.span.end,
                            );
                            issues.push(invalid_attr(
                                "#[Attribute] can only be applied to classes, not parameters",
                                loc,
                            ));
                        }
                    }
                }
                // `#[Attribute]` on a method is invalid
                for attr in method.attributes.iter() {
                    if is_attribute_class_annotation(attr) {
                        let loc = span_to_location(
                            file,
                            source,
                            source_map,
                            attr.span.start,
                            attr.span.end,
                        );
                        issues.push(invalid_attr(
                            "#[Attribute] can only be applied to classes, not methods",
                            loc,
                        ));
                    }
                }
            }
            ClassMemberKind::Property(prop) => {
                check_attribute_list(
                    &prop.attributes,
                    TARGET_PROPERTY,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                check_property_hook_attributes(
                    &prop.hooks,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                // `#[Attribute]` on a property is invalid
                for attr in prop.attributes.iter() {
                    if is_attribute_class_annotation(attr) {
                        let loc = span_to_location(
                            file,
                            source,
                            source_map,
                            attr.span.start,
                            attr.span.end,
                        );
                        issues.push(invalid_attr(
                            "#[Attribute] can only be applied to classes, not properties",
                            loc,
                        ));
                    }
                }
            }
            ClassMemberKind::ClassConst(c) => {
                check_attribute_list(
                    &c.attributes,
                    TARGET_CLASS_CONSTANT,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                // `#[Attribute]` on a class constant is invalid
                for attr in c.attributes.iter() {
                    if is_attribute_class_annotation(attr) {
                        let loc = span_to_location(
                            file,
                            source,
                            source_map,
                            attr.span.start,
                            attr.span.end,
                        );
                        issues.push(invalid_attr(
                            "#[Attribute] can only be applied to classes, not constants",
                            loc,
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

/// Check attribute placement on an interface (interfaces can't be attribute classes).
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_interface_attributes(
    decl: &InterfaceDecl,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    for attr in decl.attributes.iter() {
        if !is_attribute_class_annotation(attr) {
            continue;
        }
        let loc = span_to_location(file, source, source_map, attr.span.start, attr.span.end);
        issues.push(invalid_attr("Interfaces cannot be attribute classes", loc));
    }
    // Also check method, method-parameter, and constant attributes inside the interface.
    for member in decl.body.members.iter() {
        match &member.kind {
            ClassMemberKind::Method(method) => {
                check_attribute_list(
                    &method.attributes,
                    TARGET_METHOD,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                for param in method.params.iter() {
                    check_attribute_list(
                        &param.attributes,
                        TARGET_PARAMETER,
                        db,
                        file,
                        source,
                        source_map,
                        issues,
                        record_refs,
                        all_symbols.as_deref_mut(),
                    );
                }
            }
            ClassMemberKind::ClassConst(c) => {
                check_attribute_list(
                    &c.attributes,
                    TARGET_CLASS_CONSTANT,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
            }
            _ => {}
        }
    }
}

/// Check attribute placement on a trait (traits can't be attribute classes).
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_trait_attributes(
    decl: &TraitDecl,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    for attr in decl.attributes.iter() {
        if !is_attribute_class_annotation(attr) {
            continue;
        }
        let loc = span_to_location(file, source, source_map, attr.span.start, attr.span.end);
        issues.push(invalid_attr("Traits cannot be attribute classes", loc));
    }
    // Also validate attributes on trait members
    for member in decl.body.members.iter() {
        match &member.kind {
            ClassMemberKind::Method(method) => {
                check_attribute_list(
                    &method.attributes,
                    TARGET_METHOD,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                for param in method.params.iter() {
                    check_attribute_list(
                        &param.attributes,
                        TARGET_PARAMETER,
                        db,
                        file,
                        source,
                        source_map,
                        issues,
                        record_refs,
                        all_symbols.as_deref_mut(),
                    );
                }
            }
            ClassMemberKind::Property(prop) => {
                check_attribute_list(
                    &prop.attributes,
                    TARGET_PROPERTY,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                check_property_hook_attributes(
                    &prop.hooks,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
            }
            _ => {}
        }
    }
}

/// Check attribute placement on an enum (enums can't be attribute classes).
/// Also validates attributes on enum methods and cases — `#[SomeAttr]` on a
/// `case` is validated against `Attribute::TARGET_CLASS_CONSTANT`, matching
/// how PHP itself treats enum cases as class-constant-like targets.
#[allow(clippy::too_many_arguments)]
pub(crate) fn check_enum_attributes(
    decl: &EnumDecl,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    for attr in decl.attributes.iter() {
        if !is_attribute_class_annotation(attr) {
            continue;
        }
        let loc = span_to_location(file, source, source_map, attr.span.start, attr.span.end);
        issues.push(invalid_attr("Enums cannot be attribute classes", loc));
    }
    check_attribute_list(
        &decl.attributes,
        TARGET_CLASS,
        db,
        file,
        source,
        source_map,
        issues,
        record_refs,
        all_symbols.as_deref_mut(),
    );
    for member in decl.body.members.iter() {
        match &member.kind {
            EnumMemberKind::Method(method) => {
                check_attribute_list(
                    &method.attributes,
                    TARGET_METHOD,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
                for param in method.params.iter() {
                    check_attribute_list(
                        &param.attributes,
                        TARGET_PARAMETER,
                        db,
                        file,
                        source,
                        source_map,
                        issues,
                        record_refs,
                        all_symbols.as_deref_mut(),
                    );
                }
            }
            EnumMemberKind::Case(case) => {
                check_attribute_list(
                    &case.attributes,
                    TARGET_CLASS_CONSTANT,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
            }
            EnumMemberKind::ClassConst(c) => {
                check_attribute_list(
                    &c.attributes,
                    TARGET_CLASS_CONSTANT,
                    db,
                    file,
                    source,
                    source_map,
                    issues,
                    record_refs,
                    all_symbols.as_deref_mut(),
                );
            }
            _ => {}
        }
    }
}

/// Validate attributes on a property's PHP 8.4 hooks (`get`/`set`) and each
/// hook's own parameters. PHP reflects hooks as methods (`ReflectionProperty::
/// getHooks()` returns `ReflectionMethod`s), so a hook's own attributes are
/// validated against `TARGET_METHOD` and a hook parameter's against
/// `TARGET_PARAMETER`, matching every other method/parameter attribute site.
#[allow(clippy::too_many_arguments)]
fn check_property_hook_attributes(
    hooks: &[PropertyHook],
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    for hook in hooks.iter() {
        check_attribute_list(
            &hook.attributes,
            TARGET_METHOD,
            db,
            file,
            source,
            source_map,
            issues,
            record_refs,
            all_symbols.as_deref_mut(),
        );
        for param in hook.params.iter() {
            check_attribute_list(
                &param.attributes,
                TARGET_PARAMETER,
                db,
                file,
                source,
                source_map,
                issues,
                record_refs,
                all_symbols.as_deref_mut(),
            );
        }
    }
}

/// Emit `ParentNotFound` for any `parent::class` expression found in a flat
/// attribute argument list when the containing class has no parent.
pub(crate) fn check_parent_in_class_attrs(
    attrs: &[Attribute],
    has_parent: bool,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
) {
    if has_parent {
        return;
    }
    use php_ast::owned::ExprKind;
    for attr in attrs {
        for arg in attr.args.iter() {
            if let ExprKind::ClassConstAccess(cca) = &arg.value.kind {
                if let ExprKind::Identifier(id) = &cca.class.kind {
                    if id.as_ref().eq_ignore_ascii_case("parent") {
                        let loc = span_to_location(
                            file,
                            source,
                            source_map,
                            cca.class.span.start,
                            cca.class.span.end,
                        );
                        issues.push(Issue::new(IssueKind::ParentNotFound, loc));
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Cross-file attribute list validation
// ---------------------------------------------------------------------------

/// Validate a list of `#[...]` attributes applied to a PHP element with the
/// given `target_flag` (one of the `TARGET_*` constants above).
///
/// For each attribute in the list:
/// 1. Looks up the attribute class. If not found or not an attribute class,
///    emits `InvalidAttribute`.
/// 2. If found and has a target mask, checks that `target_flag` is set.
/// 3. Checks for duplicate non-repeatable attributes.
#[allow(clippy::too_many_arguments)]
fn check_attribute_list(
    attrs: &[Attribute],
    target_flag: i64,
    db: &dyn MirDatabase,
    file: &Arc<str>,
    source: &str,
    source_map: &SourceMap,
    issues: &mut Vec<Issue>,
    record_refs: bool,
    mut all_symbols: Option<&mut Vec<crate::symbol::ResolvedSymbol>>,
) {
    let mut seen_fqcns: Vec<(String, u32)> = Vec::new(); // (fqcn, span.start)

    for attr in attrs {
        // Skip the `Attribute` annotation itself — it is validated elsewhere.
        if is_attribute_class_annotation(attr) {
            continue;
        }

        let fqcn = resolve_attr_name(db, file.as_ref(), attr);
        let loc = span_to_location(file, source, source_map, attr.span.start, attr.span.end);

        // A class named only inside an attribute constructor argument (e.g.
        // `#[Route(Target::class)]`) is a real reference to that class — record it,
        // or a class reachable only through an attribute argument is falsely
        // flagged UnusedClass.
        if record_refs {
            for arg in attr.args.iter() {
                record_class_refs_in_expr(&arg.value, db, file, source, source_map);
            }
        }

        let class_like = find_class_like(db, Fqcn::from_str(db, &fqcn));
        match class_like {
            None => {
                // Class not found — emit UndefinedAttributeClass.
                issues.push(Issue::new(
                    IssueKind::UndefinedAttributeClass { name: fqcn.clone() },
                    loc.clone(),
                ));
            }
            Some(cl) => {
                // `#[MyAttr(...)]` is a real reference to MyAttr — record it, or a
                // class used only via attribute annotations elsewhere is falsely
                // flagged UnusedClass.
                if record_refs {
                    // Use the name token's own span, not the whole `#[Attr(...)]` —
                    // otherwise a find-references hit reports the full attribute
                    // (name and args), and a cursor anywhere inside the argument
                    // list resolves to this ClassReference symbol even when it
                    // isn't actually over the class name.
                    let name_span = attr.name.span;
                    let (line, col_start) = offset_to_line_col(source, name_span.start, source_map);
                    let (line_end, col_end) = offset_to_line_col(source, name_span.end, source_map);
                    db.record_reference_location(crate::db::RefLoc {
                        symbol_key: Arc::from(format!("cls:{fqcn}")),
                        file: file.clone(),
                        line,
                        col_start,
                        col_end: crate::diagnostics::clamp_col_end(
                            line, line_end, col_start, col_end,
                        ),
                    });
                    // Without this, hover/go-to-definition on the attribute class name
                    // in `#[MyAttr]` resolved nothing, unlike every other class-name
                    // position — the same gap already fixed for `Foo::class`.
                    if let Some(symbols) = all_symbols.as_deref_mut() {
                        symbols.push(crate::symbol::ResolvedSymbol {
                            file: file.clone(),
                            span: name_span,
                            expr_span: None,
                            kind: crate::symbol::ReferenceKind::ClassReference(Arc::from(
                                fqcn.as_str(),
                            )),
                            resolved_type: mir_types::Type::single(
                                mir_types::Atomic::TClassString(None),
                            ),
                        });
                    }
                }
                // Check for case mismatch between the written attribute name and canonical.
                if let Some((used, canonical)) =
                    crate::fqcn_case_mismatch(&fqcn, cl.fqcn().as_ref())
                {
                    issues.push(Issue::new(
                        IssueKind::WrongCaseClass { used, canonical },
                        loc.clone(),
                    ));
                }

                // Only plain `Class` entities can be attribute classes.
                use crate::db::ClassLike;
                let maybe_flags = match &cl {
                    ClassLike::Class(c) => c.attribute_flags,
                    // Interfaces, traits, enums cannot be attribute classes.
                    _ => None,
                };

                match maybe_flags {
                    None => {
                        // Class has no `#[Attribute]` annotation → not an attribute class.
                        let short = attr.name.parts.last().map(|p| p.as_ref()).unwrap_or(&fqcn);
                        issues.push(invalid_attr(
                            format!("Class {short} does not have an #[Attribute] annotation"),
                            loc.clone(),
                        ));
                    }
                    Some(flags) => {
                        // Check target mismatch (skip for TARGET_ALL = 63).
                        if flags != ATTR_TARGET_ALL && (flags & target_flag) == 0 {
                            let short = attr.name.parts.last().map(|p| p.as_ref()).unwrap_or(&fqcn);
                            issues.push(invalid_attr(
                                format!("Attribute {short} cannot be used on this target"),
                                loc.clone(),
                            ));
                        }

                        // Check repeat (IS_REPEATABLE = 64).
                        if (flags & ATTR_IS_REPEATABLE) == 0 {
                            if let Some((_prev_fqcn, prev_start)) =
                                seen_fqcns.iter().find(|(f, _)| f == &fqcn)
                            {
                                let prev_loc = span_to_location(
                                    file,
                                    source,
                                    source_map,
                                    *prev_start,
                                    *prev_start,
                                );
                                let short =
                                    attr.name.parts.last().map(|p| p.as_ref()).unwrap_or(&fqcn);
                                // Emit on the first occurrence (prev_loc), matching Psalm's behavior.
                                issues.push(invalid_attr(
                                    format!("Attribute {short} is not repeatable"),
                                    prev_loc,
                                ));
                                // Also emit on the duplicate occurrence.
                                issues.push(invalid_attr(
                                    format!("Attribute {short} is not repeatable"),
                                    loc.clone(),
                                ));
                            }
                        }
                    }
                }

                // Record this attribute to detect future repeats.
                seen_fqcns.push((fqcn, attr.span.start));
            }
        }
    }
}
