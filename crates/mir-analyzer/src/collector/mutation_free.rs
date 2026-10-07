use std::collections::HashSet;
use std::ops::ControlFlow;
use std::sync::Arc;

use mir_codebase::definitions::{MemberMap, MethodDef};
use php_ast::ast::{UnaryPostfixOp, UnaryPrefixOp};
use php_ast::owned::visitor::OwnedVisitor;
use php_ast::owned::{ClassMember, ClassMemberKind, Expr, ExprKind, MethodDecl, Stmt, StmtKind};

/// Side-effect-free builtins with no by-ref parameters.
const PURE_BUILTINS: &[&str] = &[
    "abs",
    "array_key_exists",
    "array_keys",
    "array_merge",
    "array_values",
    "count",
    "get_class",
    "implode",
    "in_array",
    "intdiv",
    "is_array",
    "is_bool",
    "is_int",
    "is_null",
    "is_numeric",
    "is_object",
    "is_string",
    "max",
    "min",
    "sizeof",
    "sprintf",
    "str_contains",
    "str_ends_with",
    "str_starts_with",
    "strlen",
    "strtolower",
    "strtoupper",
    "substr",
    "trim",
];

/// Marks the methods of one class whose bodies provably never write a
/// property, so a call to them cannot stale the receiver's property narrowing.
///
/// A method qualifies when it only writes locals and only calls allow-listed
/// pure builtins or other qualifying methods of the same class through `$this`,
/// `self` or `static`. Subclass overrides are not considered.
pub(super) fn infer_mutation_free(
    members: &[ClassMember],
    own_methods: &mut MemberMap<Arc<MethodDef>>,
) {
    let mut candidates: Vec<(String, &MethodDecl)> = members
        .iter()
        .filter_map(|m| match &m.kind {
            ClassMemberKind::Method(m) if has_inferable_signature(m, own_methods) => {
                Some((crate::util::php_ident_lowercase(m.name.as_deref()?), m))
            }
            _ => None,
        })
        .collect();

    // Greatest fixpoint: drop methods that call a non-qualifying one until stable,
    // so mutually recursive clean methods still qualify.
    loop {
        let safe: HashSet<&str> = candidates.iter().map(|(k, _)| k.as_str()).collect();
        let before = candidates.len();
        let keep: Vec<bool> = candidates
            .iter()
            .map(|(_, m)| body_writes_no_properties(m, &safe))
            .collect();
        let mut keep = keep.into_iter();
        candidates.retain(|_| keep.next().unwrap_or(false));
        if candidates.len() == before {
            break;
        }
    }

    for (key, _) in &candidates {
        if let Some(def) = own_methods.get_mut(key.as_str()) {
            Arc::make_mut(def).inferred_mutation_free = true;
        }
    }
}

fn has_inferable_signature(m: &MethodDecl, own_methods: &MemberMap<Arc<MethodDef>>) -> bool {
    let Some(name) = m.name.as_deref() else {
        return false;
    };
    m.body.is_some()
        && !m.by_ref
        && m.params.iter().all(|p| !p.by_ref)
        && own_methods
            .get(crate::util::php_ident_lowercase(name).as_str())
            .is_some_and(|d| !d.is_impure && d.self_out.is_none())
}

fn body_writes_no_properties(m: &MethodDecl, safe: &HashSet<&str>) -> bool {
    let Some(body) = &m.body else {
        return false;
    };
    let mut scan = BodyScan { safe, clean: true };
    body.stmts
        .iter()
        .all(|s| scan.visit_stmt(s).is_continue() && scan.clean)
}

struct BodyScan<'a> {
    safe: &'a HashSet<&'a str>,
    clean: bool,
}

impl BodyScan<'_> {
    fn reject(&mut self) -> ControlFlow<()> {
        self.clean = false;
        ControlFlow::Break(())
    }
}

/// A write target that cannot reach a property: a local variable, an element
/// of one, or a destructuring pattern over those.
fn is_local_target(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Variable(name) => name.trim_start_matches('$') != "this",
        ExprKind::ArrayAccess(a) => is_local_target(&a.array),
        ExprKind::Parenthesized(inner) => is_local_target(inner),
        ExprKind::Array(elements) => elements
            .iter()
            .all(|el| !el.by_ref && !el.unpack && is_local_target(&el.value)),
        ExprKind::Omit => true,
        _ => false,
    }
}

fn is_property_rooted(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::PropertyAccess(_)
        | ExprKind::NullsafePropertyAccess(_)
        | ExprKind::StaticPropertyAccess(_)
        | ExprKind::StaticPropertyAccessDynamic { .. } => true,
        ExprKind::ArrayAccess(a) => is_property_rooted(&a.array),
        ExprKind::Parenthesized(inner) => is_property_rooted(inner),
        _ => false,
    }
}

fn is_pure_builtin(name: &str) -> bool {
    PURE_BUILTINS.contains(&name.trim_start_matches('\\').to_ascii_lowercase().as_str())
}

fn is_self_reference(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Identifier(n)
        if n.eq_ignore_ascii_case("self") || n.eq_ignore_ascii_case("static"))
}

impl OwnedVisitor for BodyScan<'_> {
    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Unset(targets) if !targets.iter().all(is_local_target) => self.reject(),
            StmtKind::Foreach(fe)
                if !is_local_target(&fe.value)
                    || fe.key.as_ref().is_some_and(|k| !is_local_target(k)) =>
            {
                self.reject()
            }
            StmtKind::Global(_)
            | StmtKind::StaticVar(_)
            | StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_)
            | StmtKind::Enum(_) => self.reject(),
            _ => crate::recursion::walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        let allowed = match &expr.kind {
            ExprKind::Assign(a) => !a.by_ref && is_local_target(&a.target),
            ExprKind::UnaryPrefix(u) => {
                !matches!(
                    u.op,
                    UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement
                ) || is_local_target(&u.operand)
            }
            ExprKind::UnaryPostfix(u) => {
                !matches!(
                    u.op,
                    UnaryPostfixOp::PostIncrement | UnaryPostfixOp::PostDecrement
                ) || is_local_target(&u.operand)
            }
            ExprKind::FunctionCall(c) => {
                matches!(&c.name.kind, ExprKind::Identifier(n) if is_pure_builtin(n))
                    && c.args.iter().all(|a| {
                        !a.unpack
                            && !a.by_ref
                            && a.value.as_ref().is_none_or(|v| !is_property_rooted(v))
                    })
            }
            ExprKind::MethodCall(c) => {
                matches!(&c.object.kind, ExprKind::Variable(v) if v.trim_start_matches('$') == "this")
                    && matches!(&c.method.kind, ExprKind::Identifier(n)
                        if self.safe.contains(crate::util::php_ident_lowercase(n).as_str()))
            }
            ExprKind::StaticMethodCall(c) => {
                is_self_reference(&c.class)
                    && matches!(&c.method.kind, ExprKind::Identifier(n)
                        if self.safe.contains(crate::util::php_ident_lowercase(n).as_str()))
            }
            ExprKind::NullsafeMethodCall(_)
            | ExprKind::StaticDynMethodCall(_)
            | ExprKind::New(_)
            | ExprKind::Clone(_)
            | ExprKind::CloneWith(..)
            | ExprKind::Include(..)
            | ExprKind::Eval(_)
            | ExprKind::ShellExec(_)
            | ExprKind::VariableVariable(_)
            | ExprKind::Closure(_)
            | ExprKind::ArrowFunction(_)
            | ExprKind::AnonymousClass(_)
            | ExprKind::CallableCreate(_) => false,
            _ => true,
        };
        if !allowed {
            return self.reject();
        }
        crate::recursion::walk_expr(self, expr)
    }
}
