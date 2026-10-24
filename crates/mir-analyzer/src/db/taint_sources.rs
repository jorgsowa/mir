//! Inference-free check for which user functions and methods can return data
//! read from a taint source, and which properties a class can store such data
//! in, directly or through calls whose target resolves from declared types
//! alone. Only these need a taint summary when no argument is tainted.

use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::FunctionRefExt as _;
use crate::ast_ext::NameBareExt as _;
use crate::ast_ext::NameResolveExt as _;
use std::ops::ControlFlow;
use std::sync::Arc;

use mir_types::Name;
use php_ast::owned::visitor::OwnedVisitor;
use php_ast::owned::{ClassMemberKind, EnumMemberKind, Expr, ExprKind, MethodDecl, StmtKind};
use rustc_hash::FxHashMap;

use crate::db::{MirDatabase, SourceFile};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Callee {
    Function(Name),
    Method(Name, Name),
    /// A property read, keyed like [`prop_key`].
    Property(Name, Name),
}

/// What a value may be derived from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ValueFacts {
    reads_source: bool,
    /// Derived from a parameter.
    reads_param: bool,
    /// Derived from a property of `$this`, which is followed per object.
    reads_prop: bool,
    callees: Vec<Callee>,
}

impl ValueFacts {
    fn is_empty(&self) -> bool {
        !self.reads_source && !self.reads_param && !self.reads_prop && self.callees.is_empty()
    }

    fn merge(&mut self, other: &ValueFacts) {
        self.reads_source |= other.reads_source;
        self.reads_param |= other.reads_param;
        self.reads_prop |= other.reads_prop;
        for callee in &other.callees {
            self.push(callee.clone());
        }
    }

    fn push(&mut self, callee: Callee) {
        if !self.callees.contains(&callee) {
            self.callees.push(callee);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DeclFacts {
    /// Anything the body reads, not only what it returns.
    body: ValueFacts,
    /// Values written to `$this->prop` or a static property of the class.
    prop_writes: Vec<(Name, ValueFacts)>,
    /// Calls to the class's own methods `(class, lowercased method)` that pass a parameter's data.
    own_param_calls: Vec<(Name, Name)>,
    /// Some `return` may carry a parameter's data.
    returns_param: bool,
    /// Some `return` may carry a property of `$this`.
    returns_prop: bool,
    /// The body has a sink, or a call this scan can't resolve.
    may_reach_sink: bool,
}

impl DeclFacts {
    fn is_empty(&self) -> bool {
        self.body.is_empty()
            && self.prop_writes.is_empty()
            && self.own_param_calls.is_empty()
            && !self.returns_param
            && !self.returns_prop
            && !self.may_reach_sink
    }
}

/// `prop` for an instance property, `$prop` for a static one.
pub(crate) fn prop_key(prop: &str, is_static: bool) -> Name {
    if is_static {
        Name::new(&format!("${prop}"))
    } else {
        Name::new(prop)
    }
}

/// A function's FQN, or a class and its lowercased method name.
type DeclKey = (Name, Option<Name>);

/// Facts per declaration that reads anything worth following.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FileSourceFacts {
    decls: FxHashMap<DeclKey, DeclFacts>,
    /// Values top-level code assigns to global variables.
    globals: Vec<(Name, ValueFacts)>,
}

#[salsa::tracked]
pub(crate) fn file_source_facts(db: &dyn MirDatabase, file: SourceFile) -> Arc<FileSourceFacts> {
    let path = Arc::clone(file.path(db));
    // Untracked: a memoized `parse_file` would pin a second AST of every callee file for the run.
    let parsed = crate::recursion::parse(file.text(db).as_ref());
    let mut facts = FileSourceFacts::default();
    crate::body_analysis::for_each_file_scope_decl(&parsed.program.stmts, &mut |stmt| {
        let _scope = crate::db::ns_scope::enter(db, &path, stmt.span.start);
        let resolve = |name: &str| Name::new(&crate::db::resolve_name(db, &path, name));
        match &stmt.kind {
            StmtKind::Function(decl) => {
                let fqn = resolve(decl.name.as_str().unwrap_or(""));
                let mut scan = Scan::new(db, &path, None, None);
                scan.seed_params(&decl.params);
                let _ = crate::recursion::walk_block(&mut scan, &decl.body);
                if !scan.facts.is_empty() {
                    facts.decls.insert((fqn, None), scan.facts);
                }
            }
            StmtKind::Class(decl) => {
                let Some(name) = decl.name.as_ref().and_then(|n| n.as_str()) else {
                    return;
                };
                let fqcn = resolve(name);
                let parent = decl
                    .extends
                    .as_ref()
                    .map(|p| resolve(&crate::parser::name_to_string_owned(p)));
                for member in decl.body.members.iter() {
                    if let ClassMemberKind::Method(m) = &member.kind {
                        scan_method(db, &path, fqcn, parent, m, &mut facts);
                    }
                }
            }
            StmtKind::Trait(decl) => {
                let fqcn = resolve(decl.name.as_str().unwrap_or(""));
                for member in decl.body.members.iter() {
                    if let ClassMemberKind::Method(m) = &member.kind {
                        scan_method(db, &path, fqcn, None, m, &mut facts);
                    }
                }
            }
            StmtKind::Enum(decl) => {
                let fqcn = resolve(decl.name.as_str().unwrap_or(""));
                for member in decl.body.members.iter() {
                    if let EnumMemberKind::Method(m) = &member.kind {
                        scan_method(db, &path, fqcn, None, m, &mut facts);
                    }
                }
            }
            _ => {}
        }
    });
    let mut top_level = Scan::new(db, &path, None, None);
    scan_top_level(&mut top_level, &parsed.program.stmts);
    facts.globals = top_level
        .local_values
        .into_iter()
        .filter(|(_, value)| value.reads_source || !value.callees.is_empty())
        .map(|(name, value)| (Name::new(&name), value))
        .collect();
    Arc::new(facts)
}

fn scan_top_level(scan: &mut Scan<'_>, stmts: &[php_ast::owned::Stmt]) {
    for stmt in stmts {
        let _scope = crate::db::ns_scope::enter(scan.db, scan.path, stmt.span.start);
        match &stmt.kind {
            StmtKind::Function(_)
            | StmtKind::Class(_)
            | StmtKind::Enum(_)
            | StmtKind::Interface(_)
            | StmtKind::Trait(_)
            | StmtKind::Use(_) => {}
            StmtKind::Namespace(ns) => {
                if let php_ast::owned::NamespaceBody::Braced(body) = &ns.body {
                    scan_top_level(scan, &body.stmts);
                }
            }
            _ => {
                let _ = scan.visit_stmt(stmt);
            }
        }
    }
}

/// Files the static `require`/`include` targets of `file` name.
#[salsa::tracked]
fn included_files(db: &dyn MirDatabase, file: SourceFile) -> Arc<[SourceFile]> {
    let text = file.text(db);
    if !crate::include_graph::may_include(text) {
        return Arc::from([]);
    }
    let parsed = crate::recursion::parse(text.as_ref());
    let path = std::path::Path::new(file.path(db).as_ref());
    crate::include_graph::static_include_paths(&parsed.program, path)
        .iter()
        .filter_map(|target| db.lookup_source_file(&target.to_string_lossy()))
        .filter(|target| *target != file)
        .collect()
}

/// Include edges between workspace files: what each file includes, and who includes it.
#[derive(PartialEq, Eq)]
struct IncludeLinks {
    includes: FxHashMap<SourceFile, Vec<SourceFile>>,
    includers: FxHashMap<SourceFile, Vec<SourceFile>>,
}

#[salsa::tracked]
fn workspace_include_links(db: &dyn MirDatabase) -> Arc<IncludeLinks> {
    let _ = db.workspace_revision().revision(db);
    let mut links = IncludeLinks {
        includes: FxHashMap::default(),
        includers: FxHashMap::default(),
    };
    for file in db.all_source_files() {
        for target in included_files(db, file).iter() {
            links.includes.entry(file).or_default().push(*target);
            links.includers.entry(*target).or_default().push(file);
        }
    }
    Arc::new(links)
}

/// Adds everything reachable from `group` through `edges` to `group`.
fn extend_reachable(group: &mut Vec<SourceFile>, edges: &FxHashMap<SourceFile, Vec<SourceFile>>) {
    let mut next = 0;
    while let Some(current) = group.get(next).copied() {
        next += 1;
        for linked in edges.get(&current).into_iter().flatten() {
            if !group.contains(linked) {
                group.push(*linked);
            }
        }
    }
}

/// `file`, the files it includes and the files that include it, plus what those
/// includers include: the files whose top-level code can share `file`'s globals.
/// Files that merely include a common file stay apart.
#[salsa::tracked]
pub(crate) fn include_group(db: &dyn MirDatabase, file: SourceFile) -> Arc<[SourceFile]> {
    let links = workspace_include_links(db);
    let mut group = vec![file];
    extend_reachable(&mut group, &links.includers);
    extend_reachable(&mut group, &links.includes);
    Arc::from(group)
}

/// Whether top-level code of `file` may store source data in the global `var`.
pub(crate) fn global_may_read_source(db: &dyn MirDatabase, file: SourceFile, var: Name) -> bool {
    let facts = file_source_facts(db, file);
    facts
        .globals
        .iter()
        .any(|(name, value)| *name == var && value_reads_source(db, file.path(db), value))
}

fn scan_method(
    db: &dyn MirDatabase,
    path: &Arc<str>,
    fqcn: Name,
    parent: Option<Name>,
    method: &MethodDecl,
    facts: &mut FileSourceFacts,
) {
    let Some(body) = &method.body else { return };
    let mut scan = Scan::new(db, path, Some(fqcn), parent);
    scan.seed_params(&method.params);
    if method
        .name
        .as_str()
        .is_some_and(|name| name.eq_ignore_ascii_case("__construct"))
    {
        for param in method
            .params
            .iter()
            .filter(|p| p.modifiers.visibility.is_some())
        {
            let key = prop_key(param.name.as_str().unwrap_or(""), false);
            let value = ValueFacts {
                reads_param: true,
                ..ValueFacts::default()
            };
            scan.facts.prop_writes.push((key, value));
        }
    }
    let _ = crate::recursion::walk_block(&mut scan, body);
    if !scan.facts.is_empty() {
        let name = crate::util::php_ident_lowercase(method.name.as_str().unwrap_or(""));
        facts
            .decls
            .insert((fqcn, Some(Name::new(&name))), scan.facts);
    }
}

struct Scan<'a> {
    db: &'a dyn MirDatabase,
    path: &'a Arc<str>,
    self_fqcn: Option<Name>,
    parent_fqcn: Option<Name>,
    locals: FxHashMap<String, Name>,
    /// What each local may hold, flow-insensitively.
    local_values: FxHashMap<String, ValueFacts>,
    facts: DeclFacts,
}

/// How one expression node relates to taint sources.
enum Inspect {
    Source,
    Callee(Callee),
    SkipChildren,
    /// Only the key operands of a read are possible sources.
    Index,
    Nothing,
}

impl<'a> Scan<'a> {
    fn new(
        db: &'a dyn MirDatabase,
        path: &'a Arc<str>,
        self_fqcn: Option<Name>,
        parent_fqcn: Option<Name>,
    ) -> Self {
        Scan {
            db,
            path,
            self_fqcn,
            parent_fqcn,
            locals: FxHashMap::default(),
            local_values: FxHashMap::default(),
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
                    .insert(param.name.as_str().unwrap_or("").to_string(), class);
            }
        }
        for param in params {
            let name = param.name.as_str().unwrap_or("").to_string();
            self.local_values.entry(name).or_default().reads_param = true;
        }
    }

    fn class_in(&self, ty: &mir_types::Type) -> Option<Name> {
        ty.types.iter().find_map(|atom| match atom {
            mir_types::Atomic::TNamedObject { fqcn, .. } => Some(Name::new(
                &crate::db::resolve_name(self.db, self.path, fqcn.as_ref()),
            )),
            mir_types::Atomic::TSelf { .. } | mir_types::Atomic::TStaticObject { .. } => {
                self.self_fqcn
            }
            _ => None,
        })
    }

    fn declared_class(&self, ty: Option<&mir_types::Type>, owner: Name) -> Option<Name> {
        ty?.types.iter().find_map(|atom| match atom {
            mir_types::Atomic::TNamedObject { fqcn, .. } => Some(Name::new(fqcn.as_ref())),
            mir_types::Atomic::TSelf { .. } | mir_types::Atomic::TStaticObject { .. } => {
                Some(owner)
            }
            _ => None,
        })
    }

    fn class_name(&self, expr: &Expr) -> Option<Name> {
        let name = expr.kind.resolve_name_ref(self.db, self.path)?;
        match name.as_str() {
            n if n.eq_ignore_ascii_case("self") || n.eq_ignore_ascii_case("static") => {
                self.self_fqcn
            }
            n if n.eq_ignore_ascii_case("parent") => self.parent_fqcn,
            n => Some(Name::new(n)),
        }
    }

    /// The class an expression evaluates to, from declared types only.
    fn class_of(&self, expr: &Expr) -> Option<Name> {
        let db = self.db;
        match &expr.kind {
            ExprKind::Variable(name) => {
                let name = name.trim_start_matches('$');
                if name == "this" {
                    self.self_fqcn
                } else {
                    self.locals.get(name).copied()
                }
            }
            ExprKind::Parenthesized(inner) => self.class_of(inner),
            ExprKind::New(n) => self.class_name(&n.class),
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
                let class = self.class_of(&pa.object)?;
                let prop = crate::expr::helpers::extract_string_from_expr(&pa.property)?;
                let (owner, def) = crate::db::find_property_in_chain(
                    db,
                    crate::db::Fqcn::from_str(db, class.as_str()),
                    &prop,
                )?;
                self.declared_class(def.ty.as_deref(), Name::new(&owner))
            }
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
                let class = self.class_of(&mc.object)?;
                self.method_return_class(class, &mc.method)
            }
            ExprKind::StaticMethodCall(smc) => {
                let class = self.class_name(&smc.class)?;
                self.method_return_class(class, &smc.method)
            }
            _ => None,
        }
    }

    fn method_return_class(&self, class: Name, method: &Expr) -> Option<Name> {
        let ExprKind::Identifier(name) = &method.kind else {
            return None;
        };
        let (owner, def) = crate::db::find_method_respecting_precedence(
            self.db,
            crate::db::Fqcn::from_str(self.db, class.as_str()),
            name.as_ref(),
        )?;
        let owner = if def.return_type.as_deref().is_some_and(|t| {
            t.types
                .iter()
                .any(|a| matches!(a, mir_types::Atomic::TStaticObject { .. }))
        }) {
            class
        } else {
            Name::new(&owner)
        };
        self.declared_class(def.return_type.as_deref(), owner)
    }

    /// The class property an expression names: `$this->p`, `$obj->p`, `self::$p`.
    fn property_of(&self, expr: &Expr) -> Option<(Name, Name)> {
        match &expr.kind {
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
                let prop = crate::expr::helpers::extract_string_from_expr(&pa.property)?;
                Some((self.class_of(&pa.object)?, prop_key(&prop, false)))
            }
            ExprKind::StaticPropertyAccess(spa) => {
                let (ExprKind::Variable(prop) | ExprKind::Identifier(prop)) = &spa.member.kind
                else {
                    return None;
                };
                let prop = prop.trim_start_matches('$');
                Some((self.class_name(&spa.class)?, prop_key(prop, true)))
            }
            _ => None,
        }
    }

    fn is_server_bag_read(
        &self,
        object: &Expr,
        method: &Expr,
        args: &[php_ast::owned::Arg],
    ) -> bool {
        let (ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa)) = &object.kind
        else {
            return false;
        };
        crate::taint::server_bag_read_key(object, method, args).is_some()
            && self.class_of(&pa.object).is_some_and(|class| {
                crate::taint::is_request_bag_property(self.db, class.as_str(), "server")
            })
    }

    fn inspect(&self, expr: &Expr) -> Inspect {
        match &expr.kind {
            ExprKind::Variable(name)
                if crate::taint::is_superglobal(name.trim_start_matches('$')) =>
            {
                Inspect::Source
            }
            ExprKind::String(s) if s.as_str().contains("php://input") => Inspect::Source,
            ExprKind::ArrayAccess(aa) if crate::taint::is_globals_superglobal_alias(aa) => {
                Inspect::Source
            }
            ExprKind::ArrayAccess(aa) if crate::taint::is_trusted_superglobal_field(aa) => {
                Inspect::SkipChildren
            }
            ExprKind::ArrayAccess(aa) => match crate::taint::server_dynamic_key(aa) {
                Some(key) if crate::taint::starts_with_header_prefix(key) => Inspect::Source,
                Some(_) => Inspect::Index,
                None => Inspect::Nothing,
            },
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) => {
                let prop = crate::expr::helpers::extract_string_from_expr(&pa.property);
                match (self.class_of(&pa.object), prop) {
                    (Some(class), Some(prop))
                        if crate::taint::is_request_bag_property(
                            self.db,
                            class.as_str(),
                            &prop,
                        ) =>
                    {
                        Inspect::Source
                    }
                    (Some(class), Some(prop)) => {
                        Inspect::Callee(Callee::Property(class, prop_key(&prop, false)))
                    }
                    _ => Inspect::Nothing,
                }
            }
            ExprKind::StaticPropertyAccess(_) => match self.property_of(expr) {
                Some((class, key)) => Inspect::Callee(Callee::Property(class, key)),
                None => Inspect::Nothing,
            },
            ExprKind::FunctionCall(fc) => {
                let Some(name) = fc.name.kind.function_ref(self.db, self.path) else {
                    return Inspect::Nothing;
                };
                let lower =
                    crate::util::php_ident_lowercase(name.global.unwrap_or(&name.qualified));
                if crate::taint::is_source_function(&lower)
                    || crate::taint::is_request_helper_source(&lower, &fc.args)
                {
                    Inspect::Source
                } else if crate::stubs::is_builtin_function(&lower) {
                    Inspect::Nothing
                } else {
                    Inspect::Callee(Callee::Function(Name::new(&name.qualified)))
                }
            }
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc)
                if self.is_server_bag_read(&mc.object, &mc.method, &mc.args) =>
            {
                match crate::taint::server_bag_read_key(&mc.object, &mc.method, &mc.args) {
                    Some(key) if crate::taint::is_request_server_key_expr(key) => Inspect::Source,
                    _ => Inspect::Index,
                }
            }
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
                match (self.class_of(&mc.object), &mc.method.kind) {
                    (Some(class), ExprKind::Identifier(name)) => {
                        let lower = crate::util::php_ident_lowercase(name.as_ref());
                        Inspect::Callee(Callee::Method(class, Name::new(&lower)))
                    }
                    _ => Inspect::Nothing,
                }
            }
            ExprKind::StaticMethodCall(smc) => {
                match (self.class_name(&smc.class), &smc.method.kind) {
                    (Some(class), ExprKind::Identifier(name)) => {
                        let lower = crate::util::php_ident_lowercase(name.as_ref());
                        if crate::taint::is_laravel_request_facade_source(class.as_str(), &lower) {
                            Inspect::Source
                        } else {
                            Inspect::Callee(Callee::Method(class, Name::new(&lower)))
                        }
                    }
                    _ => Inspect::Nothing,
                }
            }
            _ => Inspect::Nothing,
        }
    }

    fn value_facts(&self, expr: &Expr) -> ValueFacts {
        let mut scan = ValueScan {
            scan: self,
            facts: ValueFacts::default(),
        };
        let _ = scan.visit_expr(expr);
        scan.facts
    }

    fn record_assign(&mut self, target: &Expr, value: ValueFacts) {
        match &target.kind {
            ExprKind::Variable(name) => {
                let name = name.trim_start_matches('$').to_string();
                self.local_values.entry(name).or_default().merge(&value);
            }
            ExprKind::ArrayAccess(aa) => self.record_assign(&aa.array, value),
            ExprKind::Array(elements) => {
                for el in elements.iter() {
                    self.record_assign(el.value(), value.clone());
                }
            }
            ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) if !matches!(&pa.object.kind, ExprKind::Variable(v) if &**v == "this") =>
            {
                self.record_assign(&pa.object, value);
            }
            _ => {
                let Some((class, key)) = self.property_of(target) else {
                    return;
                };
                if value.is_empty() || self.self_fqcn != Some(class) {
                    return;
                }
                match self.facts.prop_writes.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, existing)) => existing.merge(&value),
                    None => self.facts.prop_writes.push((key, value)),
                }
            }
        }
    }
}

fn visit_index<V: OwnedVisitor>(visitor: &mut V, expr: &Expr) -> ControlFlow<V::Break> {
    match &expr.kind {
        ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
            for value in mc.args.iter().filter_map(|a| a.value()) {
                visitor.visit_expr(value)?;
            }
            ControlFlow::Continue(())
        }
        ExprKind::ArrayAccess(aa) => aa
            .index
            .as_deref()
            .map_or(ControlFlow::Continue(()), |i| visitor.visit_expr(i)),
        _ => ControlFlow::Continue(()),
    }
}

/// Collects what one expression may be derived from.
struct ValueScan<'s, 'a> {
    scan: &'s Scan<'a>,
    facts: ValueFacts,
}

impl OwnedVisitor for ValueScan<'_, '_> {
    type Break = ();

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let ExprKind::Variable(name) = &expr.kind {
            if let Some(local) = self.scan.local_values.get(name.trim_start_matches('$')) {
                self.facts.merge(local);
            }
        }
        if let ExprKind::PropertyAccess(pa) | ExprKind::NullsafePropertyAccess(pa) = &expr.kind {
            if matches!(&pa.object.kind, ExprKind::Variable(v) if &**v == "this") {
                self.facts.reads_prop = true;
            }
        }
        match self.scan.inspect(expr) {
            Inspect::Source => self.facts.reads_source = true,
            Inspect::Callee(callee) => self.facts.push(callee),
            Inspect::SkipChildren => return ControlFlow::Continue(()),
            Inspect::Index => return visit_index(self, expr),
            Inspect::Nothing => {}
        }
        crate::recursion::walk_expr(self, expr)
    }
}

impl OwnedVisitor for Scan<'_> {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> ControlFlow<()> {
        match &stmt.kind {
            StmtKind::Foreach(f) => {
                let value = self.value_facts(&f.expr);
                if let Some(key) = &f.key {
                    self.record_assign(key, value.clone());
                }
                self.record_assign(&f.value, value);
            }
            StmtKind::Return(Some(value)) => {
                let value = self.value_facts(value);
                self.facts.returns_param |= value.reads_param;
                self.facts.returns_prop |= value.reads_prop;
            }
            StmtKind::Echo(_) => self.facts.may_reach_sink = true,
            _ => {}
        }
        crate::recursion::walk_stmt(self, stmt)
    }

    fn visit_expr(&mut self, expr: &Expr) -> ControlFlow<()> {
        if let ExprKind::Yield(y) = &expr.kind {
            for yielded in [y.key.as_deref(), y.value.as_deref()].into_iter().flatten() {
                let value = self.value_facts(yielded);
                self.facts.returns_param |= value.reads_param;
                self.facts.returns_prop |= value.reads_prop;
            }
        }
        if let ExprKind::Assign(a) = &expr.kind {
            if let ExprKind::Variable(name) = &a.target.kind {
                if let Some(class) = self.class_of(&a.value) {
                    self.locals
                        .insert(name.trim_start_matches('$').to_string(), class);
                }
            }
            let value = self.value_facts(&a.value);
            self.record_assign(&a.target, value);
        }
        if self.is_sink(expr) {
            self.facts.may_reach_sink = true;
        }
        self.record_own_param_call(expr);
        match self.inspect(expr) {
            Inspect::Source => self.facts.body.reads_source = true,
            Inspect::Callee(callee) => self.facts.body.push(callee),
            Inspect::SkipChildren => return ControlFlow::Continue(()),
            Inspect::Index => return visit_index(self, expr),
            Inspect::Nothing => {}
        }
        crate::recursion::walk_expr(self, expr)
    }
}

impl Scan<'_> {
    fn record_own_param_call(&mut self, expr: &Expr) {
        let (class, method, args) = match &expr.kind {
            ExprKind::MethodCall(mc) if matches!(&mc.object.kind, ExprKind::Variable(v) if &**v == "this") => {
                (self.self_fqcn, &mc.method, &mc.args)
            }
            ExprKind::StaticMethodCall(smc)
                if smc.class.kind.name_bare().is_some_and(|n| {
                    ["self", "static", "parent"]
                        .iter()
                        .any(|k| n.eq_ignore_ascii_case(k))
                }) =>
            {
                (self.class_name(&smc.class), &smc.method, &smc.args)
            }
            _ => return,
        };
        let (Some(class), ExprKind::Identifier(name)) = (class, &method.kind) else {
            return;
        };
        let passes_param = args
            .iter()
            .filter_map(|a| a.value())
            .any(|value| self.value_facts(value).reads_param);
        if passes_param {
            let call = (
                class,
                Name::new(&crate::util::php_ident_lowercase(name.as_ref())),
            );
            if !self.facts.own_param_calls.contains(&call) {
                self.facts.own_param_calls.push(call);
            }
        }
    }

    /// A sink, or a call whose target (and so whether it is a sink) is unknown here.
    fn is_sink(&self, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Print(_)
            | ExprKind::Exit(_)
            | ExprKind::Include(..)
            | ExprKind::Eval(_)
            | ExprKind::ShellExec(_) => true,
            ExprKind::FunctionCall(fc) => match fc.name.kind.function_ref(self.db, self.path) {
                Some(name) => {
                    crate::taint::classify_sink(name.global.unwrap_or(&name.qualified)).is_some()
                }
                _ => true,
            },
            ExprKind::MethodCall(mc) | ExprKind::NullsafeMethodCall(mc) => {
                match (self.class_of(&mc.object), &mc.method.kind) {
                    (Some(class), ExprKind::Identifier(name)) => {
                        crate::taint::classify_method_sink(self.db, class.as_str(), name).is_some()
                    }
                    _ => true,
                }
            }
            ExprKind::StaticMethodCall(smc) => !matches!(
                (self.class_name(&smc.class), &smc.method.kind),
                (Some(_), ExprKind::Identifier(_))
            ),
            ExprKind::New(n) => self.class_name(&n.class).is_none(),
            _ => false,
        }
    }
}

/// Whether a declaration of `file` may return data read from a taint source.
#[salsa::tracked(cycle_fn = reads_source_cycle, cycle_initial = reads_source_initial)]
pub(crate) fn decl_reads_source(
    db: &dyn MirDatabase,
    file: SourceFile,
    owner: Name,
    method: Option<Name>,
) -> bool {
    let facts = file_source_facts(db, file);
    let Some(decl) = facts.decls.get(&(owner, method)) else {
        return false;
    };
    value_reads_source(db, file.path(db), &decl.body)
}

fn value_reads_source(db: &dyn MirDatabase, from: &str, value: &ValueFacts) -> bool {
    value.reads_source
        || value
            .callees
            .iter()
            .any(|callee| callee_reads_source(db, from, callee))
}

fn reads_source_initial(
    _db: &dyn MirDatabase,
    _id: salsa::Id,
    _file: SourceFile,
    _owner: Name,
    _method: Option<Name>,
) -> bool {
    false
}

fn reads_source_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &bool,
    value: bool,
    _file: SourceFile,
    _owner: Name,
    _method: Option<Name>,
) -> bool {
    value
}

fn callee_reads_source(db: &dyn MirDatabase, from: &str, callee: &Callee) -> bool {
    match callee {
        Callee::Method(class, method)
            if crate::taint::is_framework_request_source(db, class.as_str(), method.as_str()) =>
        {
            true
        }
        Callee::Function(name) => {
            let name = name.as_str();
            let candidates = [name, name.rsplit('\\').next().unwrap_or(name)];
            candidates
                .iter()
                .any(|candidate| function_reads_source(db, from, candidate))
        }
        Callee::Method(class, method) => {
            method_reads_source(db, from, class.as_str(), method.as_str())
        }
        Callee::Property(class, key) => {
            !class_property_writers(db, from, class.as_str(), *key).is_empty()
        }
    }
}

/// Methods of `class` and its ancestors that may store source data in the property `key`.
pub(crate) fn class_property_writers(
    db: &dyn MirDatabase,
    from: &str,
    class: &str,
    key: Name,
) -> Vec<(Name, Name)> {
    let mut writers = Vec::new();
    for owner in crate::db::class_ancestors_by_fqcn(db, crate::db::Fqcn::from_str(db, class)).iter()
    {
        let Some(sf) = crate::db::class_like_loc(db, crate::db::Fqcn::from_str(db, owner.as_ref()))
            .map(|loc| loc.file())
        else {
            continue;
        };
        if !super::inferred_types::summarizable(db, sf, from) {
            continue;
        }
        let owner = Name::new(owner.as_ref());
        for ((decl_owner, method), decl) in file_source_facts(db, sf).decls.iter() {
            let (true, Some(method)) = (*decl_owner == owner, method) else {
                continue;
            };
            if decl.prop_writes.iter().any(|(k, _)| *k == key)
                && *prop_write_reads_source(db, sf, owner, *method, key)
            {
                writers.push((owner, *method));
            }
        }
    }
    writers
}

/// Whether a write to property `key` in method `method` of `owner` may store source data.
#[salsa::tracked(cycle_fn = prop_write_cycle, cycle_initial = prop_write_initial)]
fn prop_write_reads_source(
    db: &dyn MirDatabase,
    file: SourceFile,
    owner: Name,
    method: Name,
    key: Name,
) -> bool {
    let facts = file_source_facts(db, file);
    facts.decls.get(&(owner, Some(method))).is_some_and(|decl| {
        decl.prop_writes
            .iter()
            .any(|(k, value)| *k == key && value_reads_source(db, file.path(db), value))
    })
}

fn prop_write_initial(
    _db: &dyn MirDatabase,
    _id: salsa::Id,
    _file: SourceFile,
    _owner: Name,
    _method: Name,
    _key: Name,
) -> bool {
    false
}

#[allow(clippy::too_many_arguments)]
fn prop_write_cycle(
    _db: &dyn MirDatabase,
    _cycle: &salsa::Cycle,
    _last: &bool,
    value: bool,
    _file: SourceFile,
    _owner: Name,
    _method: Name,
    _key: Name,
) -> bool {
    value
}

/// Whether the declaration may pass a parameter's data to a sink, directly or
/// through the calls it makes.
#[salsa::tracked(cycle_fn = reads_source_cycle, cycle_initial = reads_source_initial)]
pub(crate) fn decl_may_reach_sink(
    db: &dyn MirDatabase,
    file: SourceFile,
    owner: Name,
    method: Option<Name>,
) -> bool {
    let facts = file_source_facts(db, file);
    let Some(decl) = facts.decls.get(&(owner, method)) else {
        return false;
    };
    decl.may_reach_sink
        || decl
            .body
            .callees
            .iter()
            .any(|callee| callee_may_reach_sink(db, file.path(db), callee))
}

fn callee_may_reach_sink(db: &dyn MirDatabase, from: &str, callee: &Callee) -> bool {
    match callee {
        Callee::Function(name) => {
            let name = name.as_str();
            let candidates = [name, name.rsplit('\\').next().unwrap_or(name)];
            candidates.iter().any(|candidate| {
                let here = crate::db::Fqcn::from_str(db, candidate);
                let Some(def) = crate::db::find_function_from(db, from, here) else {
                    return false;
                };
                !def.taint_sink_params.is_empty()
                    || crate::db::function_loc_from(db, from, here).is_some_and(|loc| {
                        super::inferred_types::summarizable(db, loc.file(), from)
                            && *decl_may_reach_sink(
                                db,
                                loc.file(),
                                Name::new(def.fqn.as_ref()),
                                None,
                            )
                    })
            })
        }
        Callee::Method(class, method) => {
            let Some((owner, def)) = crate::db::find_method_respecting_precedence(
                db,
                crate::db::Fqcn::from_str(db, class.as_str()),
                method.as_str(),
            ) else {
                return false;
            };
            let implementations = method_implementations(db, class.as_str(), method.as_str());
            if !implementations.is_empty() && def.taint_sink_params.is_empty() {
                return implementations.iter().any(|implementor| {
                    callee_may_reach_sink(db, from, &Callee::Method(*implementor, *method))
                });
            }
            !def.taint_sink_params.is_empty()
                || crate::db::class_like_loc(db, crate::db::Fqcn::from_str(db, owner.as_ref()))
                    .is_some_and(|loc| {
                        super::inferred_types::summarizable(db, loc.file(), from)
                            && *decl_may_reach_sink(
                                db,
                                loc.file(),
                                Name::new(owner.as_ref()),
                                Some(*method),
                            )
                    })
        }
        Callee::Property(..) => false,
    }
}

/// An interface with more implementations than this is too generic for their
/// summaries to say anything about one call.
const MAX_IMPLEMENTATIONS: usize = 8;

/// Declaring classes of the bodies `class::method_lower` may run on an
/// instance of `class`; empty unless that method is abstract there or declared
/// on an interface.
pub(crate) fn method_implementations(
    db: &dyn MirDatabase,
    class: &str,
    method_lower: &str,
) -> Arc<[Name]> {
    let Some((owner, def)) = crate::db::find_method_respecting_precedence(
        db,
        crate::db::Fqcn::from_str(db, class),
        method_lower,
    ) else {
        return Arc::from([]);
    };
    if !def.is_abstract && !crate::db::is_interface(db, owner.as_ref()) {
        return Arc::from([]);
    }
    implementations(db, Name::new(class), Name::new(method_lower)).clone()
}

/// Workspace classes under each of their ancestors, keyed by lowercased FQCN.
#[salsa::tracked]
fn workspace_subtypes(db: &dyn MirDatabase) -> Arc<FxHashMap<Name, Vec<Name>>> {
    let mut subtypes: FxHashMap<Name, Vec<Name>> = FxHashMap::default();
    for class in crate::db::workspace_classes(db).iter() {
        let here = crate::db::Fqcn::from_str(db, class.as_str());
        for ancestor in crate::db::class_ancestors_by_fqcn(db, here).iter().skip(1) {
            subtypes
                .entry(Name::new(&ancestor.to_ascii_lowercase()))
                .or_default()
                .push(*class);
        }
    }
    Arc::new(subtypes)
}

#[salsa::tracked]
fn implementations(db: &dyn MirDatabase, owner: Name, method: Name) -> Arc<[Name]> {
    let subtypes = workspace_subtypes(db);
    let mut out: Vec<Name> = Vec::new();
    let owner_key = Name::new(&owner.as_str().to_ascii_lowercase());
    for class in subtypes.get(&owner_key).into_iter().flatten() {
        let Some((declarer, def)) = crate::db::find_method_respecting_precedence(
            db,
            crate::db::Fqcn::from_str(db, class.as_str()),
            method.as_str(),
        ) else {
            continue;
        };
        if def.is_abstract || crate::db::is_interface(db, declarer.as_ref()) {
            continue;
        }
        let declarer = Name::new(declarer.as_ref());
        if !out.contains(&declarer) {
            if out.len() == MAX_IMPLEMENTATIONS {
                return Arc::from([]);
            }
            out.push(declarer);
        }
    }
    Arc::from(out)
}

/// The file and facts key of the user function `fqn` seen from `from`.
fn function_decl(db: &dyn MirDatabase, from: &str, fqn: &str) -> Option<(SourceFile, DeclKey)> {
    let here = crate::db::Fqcn::from_str(db, fqn);
    let def = crate::db::find_function_from(db, from, here)?;
    let sf = crate::db::function_loc_from(db, from, here)?.file();
    super::inferred_types::summarizable(db, sf, from)
        .then(|| (sf, (Name::new(def.fqn.as_ref()), None)))
}

/// The file and facts key of the method `method_lower` declared on `owner`.
fn method_decl(
    db: &dyn MirDatabase,
    from: &str,
    owner: &str,
    method_lower: &str,
) -> Option<(SourceFile, DeclKey)> {
    let sf = crate::db::class_like_loc(db, crate::db::Fqcn::from_str(db, owner))?.file();
    super::inferred_types::summarizable(db, sf, from)
        .then(|| (sf, (Name::new(owner), Some(Name::new(method_lower)))))
}

fn decl_fact(
    db: &dyn MirDatabase,
    decl: Option<(SourceFile, DeclKey)>,
    fact: impl Fn(&DeclFacts) -> bool,
) -> bool {
    decl.is_some_and(|(sf, key)| file_source_facts(db, sf).decls.get(&key).is_some_and(fact))
}

/// Whether some `return` of the user function `fqn` may carry a parameter's data.
pub(crate) fn function_returns_param(db: &dyn MirDatabase, from: &str, fqn: &str) -> bool {
    decl_fact(db, function_decl(db, from, fqn), |facts| {
        facts.returns_param
    })
}

/// Whether some `return` of `owner::method_lower` may carry a parameter's data.
pub(crate) fn method_returns_param(
    db: &dyn MirDatabase,
    from: &str,
    owner: &str,
    method_lower: &str,
) -> bool {
    decl_fact(db, method_decl(db, from, owner, method_lower), |facts| {
        facts.returns_param
    })
}

/// Whether some `return` of `owner::method_lower` may carry a property of `$this`.
pub(crate) fn method_returns_prop(
    db: &dyn MirDatabase,
    from: &str,
    owner: &str,
    method_lower: &str,
) -> bool {
    decl_fact(db, method_decl(db, from, owner, method_lower), |facts| {
        facts.returns_prop
    })
}

/// Whether `owner::method_lower` may store a parameter's data in a property.
pub(crate) fn method_stores_param(
    db: &dyn MirDatabase,
    from: &str,
    owner: &str,
    method_lower: &str,
) -> bool {
    method_decl(db, from, owner, method_lower)
        .is_some_and(|(sf, (owner, method))| *decl_stores_param(db, sf, owner, method))
}

/// Whether the declaration stores a parameter's data, directly or through its own methods.
#[salsa::tracked(cycle_fn = reads_source_cycle, cycle_initial = reads_source_initial)]
fn decl_stores_param(
    db: &dyn MirDatabase,
    file: SourceFile,
    owner: Name,
    method: Option<Name>,
) -> bool {
    let facts = file_source_facts(db, file);
    let Some(decl) = facts.decls.get(&(owner, method)) else {
        return false;
    };
    decl.prop_writes.iter().any(|(_, value)| value.reads_param)
        || decl.own_param_calls.iter().any(|(class, callee)| {
            let Some((callee_owner, _)) = crate::db::find_method_in_chain(
                db,
                crate::db::Fqcn::from_str(db, class.as_str()),
                callee.as_str(),
            ) else {
                return false;
            };
            method_decl(db, file.path(db), callee_owner.as_ref(), callee.as_str())
                .is_some_and(|(sf, (owner, method))| *decl_stores_param(db, sf, owner, method))
        })
}

/// Whether the user function `fqn` may pass a parameter's data to a sink.
pub(crate) fn function_may_reach_sink(db: &dyn MirDatabase, from: &str, fqn: &str) -> bool {
    function_decl(db, from, fqn)
        .is_some_and(|(sf, (owner, method))| *decl_may_reach_sink(db, sf, owner, method))
}

/// Whether `owner::method_lower` may pass a parameter's data to a sink.
pub(crate) fn method_may_reach_sink(
    db: &dyn MirDatabase,
    from: &str,
    owner: &str,
    method_lower: &str,
) -> bool {
    method_decl(db, from, owner, method_lower)
        .is_some_and(|(sf, (owner, method))| *decl_may_reach_sink(db, sf, owner, method))
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
    super::inferred_types::summarizable(db, sf, from)
        && *decl_reads_source(db, sf, Name::new(def.fqn.as_ref()), None)
}

/// Whether `method_lower` called on `class` from `from` may return source data.
pub(crate) fn method_reads_source(
    db: &dyn MirDatabase,
    from: &str,
    class: &str,
    method_lower: &str,
) -> bool {
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
    let implementations = method_implementations(db, class, method_lower);
    if !implementations.is_empty() {
        return implementations
            .iter()
            .any(|implementor| method_reads_source(db, from, implementor.as_str(), method_lower));
    }
    let Some(sf) = crate::db::class_like_loc(db, crate::db::Fqcn::from_str(db, owner.as_ref()))
        .map(|loc| loc.file())
    else {
        return false;
    };
    super::inferred_types::summarizable(db, sf, from)
        && *decl_reads_source(
            db,
            sf,
            Name::new(owner.as_ref()),
            Some(Name::new(method_lower)),
        )
}
