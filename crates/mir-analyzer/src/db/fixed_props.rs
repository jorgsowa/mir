//! Private properties that keep a literal-list default: no code in the class can change them.

use crate::ast_ext::FunctionRefExt as _;
use std::ops::ControlFlow;
use std::sync::Arc;

use mir_types::Name;
use php_ast::ast::{UnaryPrefixOp, Visibility};
use php_ast::owned::visitor::{walk_owned_expr, walk_owned_stmt, OwnedVisitor};
use php_ast::owned::{Arg, ClassMemberKind, Expr, ExprKind, Stmt, StmtKind};
use rustc_hash::FxHashSet;

use super::taint_sources::prop_key;
use crate::db::{MirDatabase, SourceFile};

/// `(class, property key)` of each private property with a list of literals as its default that
/// nothing writes, keyed like [`prop_key`].
#[salsa::tracked]
pub(crate) fn file_fixed_private_props(
    db: &dyn MirDatabase,
    file: SourceFile,
) -> Arc<FxHashSet<(Name, Name)>> {
    let path = Arc::clone(file.path(db));
    let version = crate::db::queries::db_php_version(db);
    let text = &file.text(db);
    let parsed = db.batch_parse(&path, text, version).unwrap_or_else(|| {
        Arc::new(crate::recursion::parse_for_file(
            &path,
            text.as_ref(),
            version,
        ))
    });
    let mut fixed = FxHashSet::default();
    crate::body_analysis::for_each_file_scope_decl(&parsed.program.stmts, &mut |stmt| {
        let StmtKind::Class(decl) = &stmt.kind else {
            return;
        };
        let Some(name) = decl.name.as_ref().and_then(|n| n.as_str()) else {
            return;
        };
        let _scope = crate::db::ns_scope::enter(db, &path, stmt.span.start);
        let mut writes = Writes {
            db,
            path: &path,
            props: FxHashSet::default(),
            unknown: false,
        };
        let mut candidates = Vec::new();
        for member in decl.body.members.iter() {
            match &member.kind {
                // A trait's methods and a by-ref return can change the property unseen here.
                ClassMemberKind::TraitUse(_) => return,
                ClassMemberKind::Method(m) if m.by_ref => return,
                ClassMemberKind::Property(p)
                    if p.modifiers.visibility == Some(Visibility::Private)
                        && p.default.as_ref().is_some_and(is_literal_list)
                        && p.hooks.is_empty() =>
                {
                    candidates.push(prop_key(
                        p.name.as_str().unwrap_or(""),
                        p.modifiers.is_static,
                    ));
                }
                _ => {}
            }
            let _ = writes.visit_class_member(member);
        }
        if writes.unknown {
            return;
        }
        let class = Name::new(&crate::db::prefix_file_namespace(db, &path, name));
        for key in candidates {
            if !writes.props.contains(&key) {
                fixed.insert((class, key));
            }
        }
    });
    Arc::new(fixed)
}

/// Properties any code in a class may write, by name on any object.
struct Writes<'a> {
    db: &'a dyn MirDatabase,
    path: &'a str,
    props: FxHashSet<Name>,
    /// A property with a computed name, or a `clone with`, may write any of them.
    unknown: bool,
}

impl Writes<'_> {
    fn place(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Parenthesized(inner) => self.place(inner),
            ExprKind::ArrayAccess(aa) => self.place(&aa.array),
            ExprKind::Array(items) => {
                for item in items.iter() {
                    self.place(item.value());
                }
            }
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
                if let Some(prop) = crate::expr::helpers::extract_string_from_expr(&pa.property) {
                    self.props.insert(prop_key(&prop, false));
                }
            }
            ExprKind::StaticPropertyAccess(spa) => {
                if let ExprKind::Variable(prop) | ExprKind::Identifier(prop) = &spa.member.kind {
                    self.props
                        .insert(prop_key(prop.trim_start_matches('$'), true));
                }
            }
            _ => {}
        }
    }

    fn args(&mut self, args: &[Arg], by_ref: impl Fn(usize) -> bool) {
        for (i, arg) in args.iter().enumerate() {
            if let Some(value) = arg.value().filter(|_| arg.name.is_some() || by_ref(i)) {
                self.place(value);
            }
        }
    }

    fn function_params(
        &self,
        name: &Expr,
    ) -> Option<Arc<[mir_codebase::definitions::DeclaredParam]>> {
        let name = name.kind.function_ref(self.db, self.path)?;
        let find = |fqn: &str| {
            crate::db::find_function_from(
                self.db,
                self.path,
                crate::db::Fqcn::from_str(self.db, fqn),
            )
        };
        let def = find(&name.qualified).or_else(|| name.global.and_then(find))?;
        Some(Arc::clone(&def.params))
    }
}

impl OwnedVisitor for Writes<'_> {
    type Break = ();

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        match &expr.kind {
            ExprKind::Assign(a) => {
                self.place(&a.target);
                if a.by_ref {
                    self.place(&a.value);
                }
            }
            ExprKind::UnaryPrefix(u)
                if matches!(
                    u.op,
                    UnaryPrefixOp::PreIncrement | UnaryPrefixOp::PreDecrement
                ) =>
            {
                self.place(&u.operand)
            }
            ExprKind::UnaryPostfix(u) => self.place(&u.operand),
            ExprKind::FunctionCall(fc) => match self.function_params(&fc.name) {
                Some(params) => self.args(&fc.args, |i| {
                    params
                        .get(i)
                        .or_else(|| params.last().filter(|p| p.is_variadic))
                        .is_some_and(|p| p.is_byref)
                }),
                None => self.args(&fc.args, |_| true),
            },
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
                self.args(&mc.args, |_| true)
            }
            ExprKind::StaticMethodCall(smc) => self.args(&smc.args, |_| true),
            ExprKind::StaticMethodCallDynamic(smc) => self.args(&smc.args, |_| true),
            ExprKind::New(n) => self.args(&n.args, |_| true),
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa)
                if crate::expr::helpers::extract_string_from_expr(&pa.property).is_none() =>
            {
                self.unknown = true
            }
            ExprKind::StaticPropertyAccessDynamic { .. } | ExprKind::CloneWith(..) => {
                self.unknown = true
            }
            _ => {}
        }
        walk_owned_expr(self, expr)
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Unset(exprs) => exprs.iter().for_each(|e| self.place(e)),
            StmtKind::Foreach(f) if f.by_ref => self.place(&f.expr),
            _ => {}
        }
        walk_owned_stmt(self, stmt)
    }
}

fn is_literal_list(expr: &Expr) -> bool {
    let ExprKind::Array(items) = &expr.kind else {
        return false;
    };
    items.iter().all(|item| {
        !item.is_spread()
            && !item.is_by_ref()
            && matches!(item.value().kind, ExprKind::String(_) | ExprKind::Int(_))
    })
}
