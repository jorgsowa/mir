//! Inference-free check for which user functions and methods can return data
//! read from a taint source, directly or through calls whose target resolves
//! from declared types alone. A callee outside this set needs no taint summary
//! unless its arguments are tainted.

use std::ops::ControlFlow;
use std::sync::Arc;

use php_ast::owned::visitor::OwnedVisitor;
use php_ast::owned::{ClassMemberKind, EnumMemberKind, Expr, ExprKind, MethodDecl, StmtKind};
use rustc_hash::FxHashMap;

use crate::db::{MirDatabase, SourceFile};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Callee {
    Function(Arc<str>),
    Method(Arc<str>, Arc<str>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DeclFacts {
    reads_source: bool,
    callees: Vec<Callee>,
}

/// Facts per declaration, keyed by function FQN or `Class::method` (lowercased method).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileSourceFacts {
    decls: FxHashMap<Arc<str>, DeclFacts>,
}

fn method_key(fqcn: &str, method_lower: &str) -> Arc<str> {
    Arc::from(format!("{fqcn}::{method_lower}"))
}

#[salsa::tracked]
pub(crate) fn file_source_facts(db: &dyn MirDatabase, file: SourceFile) -> Arc<FileSourceFacts> {
    let prepared = super::queries::prepare_analysis_file(db, file);
    let path = Arc::clone(prepared.path);
    let parsed = prepared.parse_result();
    let mut facts = FileSourceFacts::default();
    crate::body_analysis::for_each_file_scope_decl(&parsed.program.stmts, &mut |stmt| {
        let _scope = crate::db::ns_scope::enter(db, &path, stmt.span.start);
        let resolve = |name: &str| Arc::<str>::from(crate::db::resolve_name(db, &path, name));
        match &stmt.kind {
            StmtKind::Function(decl) => {
                let fqn = resolve(decl.name.as_deref().unwrap_or(""));
                let mut scan = Scan::new(db, &path, None, None);
                scan.seed_params(&decl.params);
                let _ = crate::recursion::walk_block(&mut scan, &decl.body);
                facts.decls.insert(fqn, scan.facts);
            }
            StmtKind::Class(decl) => {
                let Some(name) = decl.name.as_ref().and_then(|n| n.as_deref()) else {
                    return;
                };
                let fqcn = resolve(name);
                let parent = decl
                    .extends
                    .as_ref()
                    .map(|p| resolve(&crate::parser::name_to_string_owned(p)));
                for member in decl.body.members.iter() {
                    if let ClassMemberKind::Method(m) = &member.kind {
                        scan_method(db, &path, &fqcn, parent.clone(), m, &mut facts);
                    }
                }
            }
            StmtKind::Trait(decl) => {
                let fqcn = resolve(decl.name.as_deref().unwrap_or(""));
                for member in decl.body.members.iter() {
                    if let ClassMemberKind::Method(m) = &member.kind {
                        scan_method(db, &path, &fqcn, None, m, &mut facts);
                    }
                }
            }
            StmtKind::Enum(decl) => {
                let fqcn = resolve(decl.name.as_deref().unwrap_or(""));
                for member in decl.body.members.iter() {
                    if let EnumMemberKind::Method(m) = &member.kind {
                        scan_method(db, &path, &fqcn, None, m, &mut facts);
                    }
                }
            }
            _ => {}
        }
    });
    Arc::new(facts)
}

fn scan_method(
    db: &dyn MirDatabase,
    path: &Arc<str>,
    fqcn: &Arc<str>,
    parent: Option<Arc<str>>,
    method: &MethodDecl,
    facts: &mut FileSourceFacts,
) {
    let Some(body) = &method.body else { return };
    let mut scan = Scan::new(db, path, Some(fqcn.clone()), parent);
    scan.seed_params(&method.params);
    let _ = crate::recursion::walk_block(&mut scan, body);
    let name = crate::util::php_ident_lowercase(method.name.as_deref().unwrap_or(""));
    facts.decls.insert(method_key(fqcn, &name), scan.facts);
}

struct Scan<'a> {
    db: &'a dyn MirDatabase,
    path: &'a Arc<str>,
    self_fqcn: Option<Arc<str>>,
    parent_fqcn: Option<Arc<str>>,
    locals: FxHashMap<String, Arc<str>>,
    facts: DeclFacts,
}

impl<'a> Scan<'a> {
    fn new(
        db: &'a dyn MirDatabase,
        path: &'a Arc<str>,
        self_fqcn: Option<Arc<str>>,
        parent_fqcn: Option<Arc<str>>,
    ) -> Self {
        Scan {
            db,
            path,
            self_fqcn,
            parent_fqcn,
            locals: FxHashMap::default(),
            facts: DeclFacts::default(),
        }
    }

    fn seed_params(&mut self, params: &[php_ast::owned::Param]) {
        for param in params {
            let Some(hint) = &param.type_hint else {
                continue;
            };
            let ty = crate::parser::type_from_hint_owned(hint, self.self_fqcn.as_deref());
            if let Some(class) = self.class_in(&ty) {
                self.locals
                    .insert(param.name.as_deref().unwrap_or("").to_string(), class);
            }
        }
    }

    fn class_in(&self, ty: &mir_types::Type) -> Option<Arc<str>> {
        ty.types.iter().find_map(|atom| match atom {
            mir_types::Atomic::TNamedObject { fqcn, .. } => Some(Arc::from(
                crate::db::resolve_name(self.db, self.path, fqcn.as_ref()).as_str(),
            )),
            mir_types::Atomic::TSelf { .. } | mir_types::Atomic::TStaticObject { .. } => {
                self.self_fqcn.clone()
            }
            _ => None,
        })
    }

    fn declared_class(&self, ty: Option<&mir_types::Type>, owner: &Arc<str>) -> Option<Arc<str>> {
        ty?.types.iter().find_map(|atom| match atom {
            mir_types::Atomic::TNamedObject { fqcn, .. } => Some(Arc::from(fqcn.as_ref())),
            mir_types::Atomic::TSelf { .. } | mir_types::Atomic::TStaticObject { .. } => {
                Some(owner.clone())
            }
            _ => None,
        })
    }

    fn class_name(&self, expr: &Expr) -> Option<Arc<str>> {
        let ExprKind::Identifier(name) = &expr.kind else {
            return None;
        };
        match crate::util::php_ident_lowercase(name.as_ref()).as_str() {
            "self" | "static" => self.self_fqcn.clone(),
            "parent" => self.parent_fqcn.clone(),
            _ => Some(Arc::from(
                crate::db::resolve_name(self.db, self.path, name.as_ref()).as_str(),
            )),
        }
    }

    /// The class an expression evaluates to, from declared types only.
    fn class_of(&self, expr: &Expr) -> Option<Arc<str>> {
        let db = self.db;
        match &expr.kind {
            ExprKind::Variable(name) => {
                let name = name.trim_start_matches('$');
                if name == "this" {
                    self.self_fqcn.clone()
                } else {
                    self.locals.get(name).cloned()
                }
            }
            ExprKind::Parenthesized(inner) => self.class_of(inner),
            ExprKind::New(n) => self.class_name(&n.class),
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
                let class = self.class_of(&pa.object)?;
                let prop = crate::expr::helpers::extract_string_from_expr(&pa.property)?;
                let (owner, def) = crate::db::find_property_in_chain(
                    db,
                    crate::db::Fqcn::from_str(db, class.as_ref()),
                    &prop,
                )?;
                self.declared_class(def.ty.as_deref(), &owner)
            }
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
                let class = self.class_of(&mc.object)?;
                self.method_return_class(&class, &mc.method)
            }
            ExprKind::StaticMethodCall(smc) => {
                let class = self.class_name(&smc.class)?;
                self.method_return_class(&class, &smc.method)
            }
            _ => None,
        }
    }

    fn method_return_class(&self, class: &Arc<str>, method: &Expr) -> Option<Arc<str>> {
        let ExprKind::Identifier(name) = &method.kind else {
            return None;
        };
        let (owner, def) = crate::db::find_method_respecting_precedence(
            self.db,
            crate::db::Fqcn::from_str(self.db, class.as_ref()),
            name.as_ref(),
        )?;
        let owner = if def.return_type.as_deref().is_some_and(|t| {
            t.types
                .iter()
                .any(|a| matches!(a, mir_types::Atomic::TStaticObject { .. }))
        }) {
            class.clone()
        } else {
            owner
        };
        self.declared_class(def.return_type.as_deref(), &owner)
    }

    fn push(&mut self, callee: Callee) {
        if !self.facts.callees.contains(&callee) {
            self.facts.callees.push(callee);
        }
    }
}

impl OwnedVisitor for Scan<'_> {
    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if self.facts.reads_source {
            return ControlFlow::Break(());
        }
        match &expr.kind {
            ExprKind::Variable(name)
                if crate::taint::is_superglobal(name.trim_start_matches('$')) =>
            {
                self.facts.reads_source = true;
            }
            ExprKind::String(s) if s.contains("php://input") || s.contains("php://stdin") => {
                self.facts.reads_source = true;
            }
            ExprKind::Assign(a) => {
                if let ExprKind::Variable(name) = &a.target.kind {
                    if let Some(class) = self.class_of(&a.value) {
                        self.locals
                            .insert(name.trim_start_matches('$').to_string(), class);
                    }
                }
            }
            ExprKind::FunctionCall(fc) => {
                if let ExprKind::Identifier(name) = &fc.name.kind {
                    let lower = crate::util::php_ident_lowercase(name.as_ref());
                    if crate::taint::is_source_function(&lower) {
                        self.facts.reads_source = true;
                    } else if !crate::stubs::is_builtin_function(&lower) {
                        let resolved = crate::db::resolve_name(self.db, self.path, name.as_ref());
                        self.push(Callee::Function(Arc::from(resolved.as_str())));
                    }
                }
            }
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
                if let (Some(class), ExprKind::Identifier(name)) =
                    (self.class_of(&mc.object), &mc.method.kind)
                {
                    let lower = crate::util::php_ident_lowercase(name.as_ref());
                    self.push(Callee::Method(class, Arc::from(lower.as_str())));
                }
            }
            ExprKind::StaticMethodCall(smc) => {
                if let (Some(class), ExprKind::Identifier(name)) =
                    (self.class_name(&smc.class), &smc.method.kind)
                {
                    let lower = crate::util::php_ident_lowercase(name.as_ref());
                    self.push(Callee::Method(class, Arc::from(lower.as_str())));
                }
            }
            _ => {}
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// Whether the declaration `key` of `file` may return data read from a taint source.
#[salsa::tracked(cycle_fn = reads_source_cycle, cycle_initial = reads_source_initial)]
pub(crate) fn decl_reads_source(db: &dyn MirDatabase, file: SourceFile, key: Arc<str>) -> bool {
    let facts = file_source_facts(db, file);
    let Some(decl) = facts.decls.get(&key) else {
        return false;
    };
    if decl.reads_source {
        return true;
    }
    let path = file.path(db);
    decl.callees
        .iter()
        .any(|callee| callee_reads_source(db, path, callee))
}

fn reads_source_initial(
    _db: &dyn MirDatabase,
    _id: salsa::Id,
    _file: SourceFile,
    _key: Arc<str>,
) -> bool {
    false
}

fn reads_source_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &bool,
    value: bool,
    _file: SourceFile,
    _key: Arc<str>,
) -> bool {
    value
}

fn callee_reads_source(db: &dyn MirDatabase, from: &str, callee: &Callee) -> bool {
    match callee {
        Callee::Function(name) => {
            let candidates = [name.as_ref(), name.rsplit('\\').next().unwrap_or(name)];
            candidates
                .iter()
                .any(|candidate| function_reads_source(db, from, candidate))
        }
        Callee::Method(class, method) => method_reads_source(db, class, method),
    }
}

/// Whether the user function `fqn`, seen from `from`, may return source data.
pub(crate) fn function_reads_source(db: &dyn MirDatabase, from: &str, fqn: &str) -> bool {
    let here = crate::db::Fqcn::from_str(db, fqn);
    let Some(def) = crate::db::find_function_from(db, from, here) else {
        return false;
    };
    if def.is_taint_source {
        return true;
    }
    let Some(sf) = crate::db::function_loc_from(db, from, here).map(|loc| loc.file()) else {
        return false;
    };
    !crate::stubs::is_stub_path(sf.path(db).as_ref())
        && *decl_reads_source(db, sf, Arc::from(def.fqn.as_ref()))
}

/// Whether `method_lower` called on `class` may return source data.
pub(crate) fn method_reads_source(db: &dyn MirDatabase, class: &str, method_lower: &str) -> bool {
    let Some((owner, def)) = crate::db::find_method_respecting_precedence(
        db,
        crate::db::Fqcn::from_str(db, class),
        method_lower,
    ) else {
        return false;
    };
    if def.is_taint_source {
        return true;
    }
    let Some(sf) = crate::db::class_like_loc(db, crate::db::Fqcn::from_str(db, owner.as_ref()))
        .map(|loc| loc.file())
    else {
        return false;
    };
    !crate::stubs::is_stub_path(sf.path(db).as_ref())
        && *decl_reads_source(db, sf, method_key(&owner, method_lower))
}
