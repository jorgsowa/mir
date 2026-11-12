use crate::ast_ext::ArgExt as _;
use crate::ast_ext::NameBareExt as _;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::ControlFlow;
use std::sync::Arc;

use mir_codebase::definitions::{MemberMap, MethodDef, PropertyWrite};
use php_ast::ast::{AssignOp, UnaryPostfixOp, UnaryPrefixOp};
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
/// property, so a call to them cannot stale the receiver's property narrowing,
/// and records which `$this` properties the others change, when that is all
/// they can change.
///
/// A method is mutation-free when it only writes locals and only calls
/// allow-listed pure builtins or other such methods of the same class through
/// `$this`, `self` or `static`. The write summary also admits `$this` property
/// writes and calls on other objects, as long as `$this` itself never escapes.
/// Subclass overrides are not considered.
pub(super) fn infer_mutation_free(
    members: &[ClassMember],
    own_methods: &mut MemberMap<Arc<MethodDef>>,
) {
    let candidates: Vec<(String, &MethodDecl)> = members
        .iter()
        .filter_map(|m| match &m.kind {
            ClassMemberKind::Method(m) if has_inferable_signature(m, own_methods) => {
                Some((crate::util::php_ident_lowercase(m.name.as_str()?), m))
            }
            _ => None,
        })
        .collect();

    for (key, _) in qualifying(&candidates, false) {
        if let Some(def) = own_methods.get_mut(key.as_str()) {
            Arc::make_mut(def).inferred_mutation_free = true;
        }
    }
    for (key, writes) in qualifying(&candidates, true) {
        if let Some(def) = own_methods.get_mut(key.as_str()) {
            Arc::make_mut(def).inferred_this_writes = Some(
                writes
                    .into_iter()
                    .map(|(prop, write)| (Arc::from(prop), write))
                    .collect(),
            );
        }
    }
}

/// The candidates whose bodies pass the scan, each with the `$this` property writes of its
/// body and of the methods it calls.
fn qualifying(
    candidates: &[(String, &MethodDecl)],
    lenient: bool,
) -> Vec<(String, BTreeMap<String, PropertyWrite>)> {
    // Greatest fixpoint: drop methods that call a non-qualifying one until stable,
    // so mutually recursive methods still qualify.
    let mut kept: Vec<&(String, &MethodDecl)> = candidates.iter().collect();
    let mut scans: Vec<BodySummary>;
    loop {
        let safe: HashSet<&str> = kept.iter().map(|(k, _)| k.as_str()).collect();
        scans = kept
            .iter()
            .map(|(_, m)| scan_body(m, &safe, lenient))
            .collect();
        let before = kept.len();
        let mut keep = scans.iter().map(|s| s.qualifies);
        kept.retain(|_| keep.next().unwrap_or(false));
        if kept.len() == before {
            break;
        }
    }

    let index: HashMap<&str, usize> = kept
        .iter()
        .enumerate()
        .map(|(i, (k, _))| (k.as_str(), i))
        .collect();
    let mut writes: Vec<_> = scans.iter().map(|s| s.writes.clone()).collect();
    loop {
        let mut changed = false;
        for (i, scan) in scans.iter().enumerate() {
            for callee in &scan.calls {
                let Some(&j) = index.get(callee.as_str()).filter(|&&j| j != i) else {
                    continue;
                };
                for (prop, write) in writes[j].clone() {
                    changed |= record_write(&mut writes[i], prop, write);
                }
            }
        }
        if !changed {
            break;
        }
    }
    kept.iter().map(|(k, _)| k.clone()).zip(writes).collect()
}

/// Adds a write of `$this->prop`; two different kinds of write combine to `Assign`.
fn record_write(
    writes: &mut BTreeMap<String, PropertyWrite>,
    prop: String,
    write: PropertyWrite,
) -> bool {
    match writes.get_mut(&prop) {
        Some(existing) if *existing != write && *existing != PropertyWrite::Assign => {
            *existing = PropertyWrite::Assign;
            true
        }
        Some(_) => false,
        None => {
            writes.insert(prop, write);
            true
        }
    }
}

fn has_inferable_signature(m: &MethodDecl, own_methods: &MemberMap<Arc<MethodDef>>) -> bool {
    let Some(name) = m.name.as_str() else {
        return false;
    };
    m.body.is_some()
        && !m.by_ref
        && m.params.iter().all(|p| !p.by_ref)
        && own_methods
            .get(crate::util::php_ident_lowercase(name).as_str())
            .is_some_and(|d| !d.is_impure && d.self_out.is_none())
}

struct BodySummary {
    qualifies: bool,
    writes: BTreeMap<String, PropertyWrite>,
    calls: Vec<String>,
}

fn scan_body(m: &MethodDecl, safe: &HashSet<&str>, lenient: bool) -> BodySummary {
    let mut scan = BodyScan {
        safe,
        lenient,
        writes: BTreeMap::new(),
        calls: Vec::new(),
    };
    let qualifies = m
        .body
        .as_ref()
        .is_some_and(|body| body.stmts.iter().all(|s| scan.visit_stmt(s).is_continue()));
    BodySummary {
        qualifies,
        writes: scan.writes,
        calls: scan.calls,
    }
}

/// Whether a function body visibly has side effects: output, `exit`, `global` or `static` variables.
pub(crate) fn has_visible_side_effect(stmts: &[Stmt]) -> bool {
    stmts
        .iter()
        .any(|s| SideEffectScan.visit_stmt(s).is_break())
}

/// [`has_visible_side_effect`] for an arrow function's body expression.
pub(crate) fn expr_has_visible_side_effect(body: &Expr) -> bool {
    SideEffectScan.visit_expr(body).is_break()
}

struct SideEffectScan;

impl OwnedVisitor for SideEffectScan {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Echo(_)
            | StmtKind::InlineHtml(_)
            | StmtKind::Global(_)
            | StmtKind::StaticVar(_) => ControlFlow::Break(()),
            // A nested declaration runs only when called.
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_)
            | StmtKind::Enum(_) => ControlFlow::Continue(()),
            _ => crate::recursion::walk_stmt(self, stmt),
        }
    }

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        match &expr.kind {
            ExprKind::Print(_) | ExprKind::Exit(_) => ControlFlow::Break(()),
            ExprKind::Closure(_) | ExprKind::ArrowFunction(_) | ExprKind::AnonymousClass(_) => {
                ControlFlow::Continue(())
            }
            _ => crate::recursion::walk_expr(self, expr),
        }
    }
}

struct BodyScan<'a> {
    safe: &'a HashSet<&'a str>,
    /// Admits `$this` property writes and calls on other objects, recording the writes.
    lenient: bool,
    writes: BTreeMap<String, PropertyWrite>,
    calls: Vec<String>,
}

impl BodyScan<'_> {
    fn reject(&self) -> ControlFlow<()> {
        ControlFlow::Break(())
    }

    /// Whether writing `e` is allowed, recording it when it is a `$this` property.
    fn write(&mut self, e: &Expr, appends: bool) -> bool {
        if is_local_target(e) {
            return true;
        }
        let Some(prop) = this_property(e).filter(|_| self.lenient) else {
            return false;
        };
        // Appending to an element can still change the property's shape.
        let write = if appends && matches!(e.kind, ExprKind::PropertyAccess(_)) {
            PropertyWrite::Append
        } else {
            PropertyWrite::Assign
        };
        record_write(&mut self.writes, prop.to_string(), write);
        true
    }

    /// Whether `name` is a qualifying method of this class, recording the call.
    fn call(&mut self, name: &Expr) -> bool {
        let ExprKind::Identifier(n) = &name.kind else {
            return false;
        };
        let key = crate::util::php_ident_lowercase(n);
        let safe = self.safe.contains(key.as_str());
        if safe {
            self.calls.push(key);
        }
        safe
    }

    /// An unknown callee may take a `$this` property argument by reference.
    fn pass_args(&mut self, args: &[php_ast::owned::Arg]) {
        for prop in args.iter().filter_map(|a| this_property(a.value()?)) {
            record_write(&mut self.writes, prop.to_string(), PropertyWrite::Assign);
        }
    }

    fn visit_args(&mut self, args: &[php_ast::owned::Arg]) -> ControlFlow<()> {
        args.iter()
            .filter_map(|a| a.value())
            .try_for_each(|v| self.visit_expr(v))
    }

    fn visit_expr_lenient(&mut self, expr: &Expr) -> Option<ControlFlow<()>> {
        Some(match &expr.kind {
            // Any other use of `$this` lets it escape.
            ExprKind::Variable(v) if &**v == "this" => self.reject(),
            ExprKind::PropertyAccess(p) | ExprKind::NullsafePropertyAccess(p)
                if is_this(&p.object) =>
            {
                if matches!(p.property.kind, ExprKind::Identifier(_)) {
                    ControlFlow::Continue(())
                } else {
                    self.reject()
                }
            }
            ExprKind::MethodCall(c) if is_this(&c.object) => {
                if self.call(&c.method) {
                    self.visit_args(&c.args)
                } else {
                    self.reject()
                }
            }
            ExprKind::MethodCall(c) | ExprKind::NullsafeMethodCall(c) => {
                if let Some(prop) = this_property(&c.object) {
                    record_write(&mut self.writes, prop.to_string(), PropertyWrite::Inner);
                }
                self.pass_args(&c.args);
                crate::recursion::walk_expr(self, expr)
            }
            ExprKind::FunctionCall(c) => {
                let Some(name) = c.name.kind.name_bare() else {
                    return Some(self.reject());
                };
                if !is_pure_builtin(name) {
                    self.pass_args(&c.args);
                }
                self.visit_args(&c.args)
            }
            ExprKind::StaticMethodCall(c) if is_self_reference(&c.class) => {
                if self.call(&c.method) {
                    self.visit_args(&c.args)
                } else {
                    self.reject()
                }
            }
            ExprKind::StaticMethodCall(c)
                if !c
                    .class
                    .kind
                    .name_bare()
                    .is_some_and(|n| n.eq_ignore_ascii_case("parent")) =>
            {
                self.pass_args(&c.args);
                crate::recursion::walk_expr(self, expr)
            }
            ExprKind::New(n) => {
                self.pass_args(&n.args);
                crate::recursion::walk_expr(self, expr)
            }
            ExprKind::Clone(_) | ExprKind::CloneWith(..) => crate::recursion::walk_expr(self, expr),
            ExprKind::Closure(c) if c.is_static => crate::recursion::walk_expr(self, expr),
            ExprKind::ArrowFunction(f) if f.is_static => crate::recursion::walk_expr(self, expr),
            _ => return None,
        })
    }
}

fn is_this(e: &Expr) -> bool {
    matches!(&e.kind, ExprKind::Variable(v) if &**v == "this")
}

/// The property name of `$this->name` or an element of it.
fn this_property(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::PropertyAccess(p) if is_this(&p.object) => match &p.property.kind {
            ExprKind::Identifier(n) => Some(n),
            _ => None,
        },
        ExprKind::ArrayAccess(a) => this_property(&a.array),
        ExprKind::Parenthesized(inner) => this_property(inner),
        _ => None,
    }
}

/// A write target that cannot reach a property: a local variable, an element
/// of one, or a destructuring pattern over those.
fn is_local_target(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Variable(name) => &**name != "this",
        ExprKind::ArrayAccess(a) => is_local_target(&a.array),
        ExprKind::Parenthesized(inner) => is_local_target(inner),
        ExprKind::Array(elements) => elements
            .iter()
            .all(|el| !el.is_by_ref() && !el.is_spread() && is_local_target(el.value())),
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
    PURE_BUILTINS.contains(&name.to_ascii_lowercase().as_str())
}

fn is_self_reference(e: &Expr) -> bool {
    e.kind
        .name_bare()
        .is_some_and(|n| n.eq_ignore_ascii_case("self") || n.eq_ignore_ascii_case("static"))
}

impl OwnedVisitor for BodyScan<'_> {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Unset(targets) if !targets.iter().all(|t| self.write(t, false)) => {
                self.reject()
            }
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
        if self.lenient {
            if let Some(flow) = self.visit_expr_lenient(expr) {
                return flow;
            }
        }
        let allowed = match &expr.kind {
            ExprKind::Assign(a) => !a.by_ref && self.write(&a.target, a.op == AssignOp::Concat),
            ExprKind::UnaryPrefix(u) => {
                !matches!(
                    u.op,
                    UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement
                ) || self.write(&u.operand, false)
            }
            ExprKind::UnaryPostfix(u) => {
                !matches!(
                    u.op,
                    UnaryPostfixOp::PostIncrement | UnaryPostfixOp::PostDecrement
                ) || self.write(&u.operand, false)
            }
            ExprKind::FunctionCall(c) => {
                c.name.kind.name_bare().is_some_and(is_pure_builtin)
                    && c.args.iter().all(|a| {
                        !a.unpack()
                            && !matches!(a.kind, php_ast::owned::ArgKind::ByRef(_))
                            && a.value().is_none_or(|v| !is_property_rooted(v))
                    })
            }
            ExprKind::MethodCall(c) => {
                matches!(&c.object.kind, ExprKind::Variable(v) if &**v == "this")
                    && self.call(&c.method)
            }
            ExprKind::StaticMethodCall(c) => is_self_reference(&c.class) && self.call(&c.method),
            ExprKind::NullsafeMethodCall(_)
            | ExprKind::StaticMethodCallDynamic(_)
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
