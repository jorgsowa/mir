/// Analysis dataflow and lexical scope state — carries type state through statement/expression analysis.
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

use mir_types::{Name, Type};

/// FQCNs known to exist in the current branch due to a `class_exists()` /
/// `interface_exists()` / `trait_exists()` guard.  Not Arc-wrapped — it is
/// small and branch-local (cleared at merge unless one branch diverges).
type ClassExistsGuards = FxHashSet<Arc<str>>;

/// A dead write: `(variable, line, col_start, line_end, col_end)`.
type DeadWrite = (Name, u32, u16, u32, u16);

/// Append `src` dead writes onto `dst`, skipping entries already present.
///
/// Critical for bounding memory in `merge_branches`: both branch contexts are
/// derived from `pre`, so each already contains all of `pre`'s dead writes.
/// Naively concatenating them onto a `pre`-derived `dst` re-includes `pre`'s
/// entries on every merge — under nested-loop fixpoint analysis the `Vec` then
/// grows multiplicatively (≈3× per merge → exponential, reaching gigabytes).
/// A dead write is uniquely identified by its `(variable, location)` tuple, so
/// deduplication is also semantically correct (one diagnostic per location).
fn extend_dead_writes_dedup(dst: &mut Vec<DeadWrite>, src: Vec<DeadWrite>) {
    if src.is_empty() {
        return;
    }
    let mut seen: FxHashSet<DeadWrite> = dst.iter().copied().collect();
    for dw in src {
        if seen.insert(dw) {
            dst.push(dw);
        }
    }
}

// ---------------------------------------------------------------------------
// FlowState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct FlowState {
    /// Types of variables at this point in execution.
    /// Arc-wrapped for COW semantics: fork() is O(1); mutations trigger a copy only on first write.
    /// Values are Arc<Type> so ptr_eq short-circuits merge_branches for unchanged vars.
    pub vars: Arc<FxHashMap<Name, Arc<Type>>>,

    /// Variables that are definitely assigned at this point.
    pub assigned_vars: Arc<FxHashSet<Name>>,

    /// Variables that *might* be assigned (e.g. only in one if branch).
    pub possibly_assigned_vars: Arc<FxHashSet<Name>>,

    /// The class in whose body we are analysing (`self`).
    pub self_fqcn: Option<Arc<str>>,

    /// The parent class (`parent`).
    pub parent_fqcn: Option<Arc<str>>,

    /// Late-static-binding class (`static`).
    pub static_fqcn: Option<Arc<str>>,

    /// Declared return type for the current function/method.
    pub fn_return_type: Option<Type>,

    /// Declared exception types for the current function/method (@throws).
    pub fn_declared_throws: Arc<[Arc<str>]>,

    /// Whether we are currently inside a loop.
    pub inside_loop: bool,

    /// Whether we are currently inside a finally block.
    pub inside_finally: bool,

    /// Whether the current function body contains a `yield` expression, making
    /// it a generator. Set via pre-scan before statement analysis begins so
    /// that early `return;` before the first yield is not misreported.
    pub is_generator: bool,

    /// Whether we are inside a constructor.
    pub inside_constructor: bool,

    /// Whether we are inside a @pure function/method body.
    pub is_in_pure_fn: bool,

    /// Whether we are inside a static method body.
    pub inside_static_method: bool,

    /// Whether `strict_types=1` is declared for this file.
    pub strict_types: bool,

    /// Variables that carry tainted (user-controlled) values at this point.
    /// Used by taint analysis (M19).
    pub tainted_vars: FxHashSet<Name>,

    /// Variables that have been read at least once in this scope.
    /// Used by UnusedParam detection (M18).
    pub read_vars: FxHashSet<Name>,

    /// Names of function/method parameters in this scope (stripped of `$`).
    /// Used to exclude parameters from UnusedVariable detection.
    /// Arc-shared — set once at context construction, never mutated during analysis.
    pub param_names: Arc<FxHashSet<Name>>,

    /// Names of by-reference parameters in this scope (stripped of `$`).
    /// Assigning to these is externally observable, so it counts as usage.
    /// Arc-shared — set once at context construction, never mutated during analysis.
    pub byref_param_names: Arc<FxHashSet<Name>>,

    /// Whether every execution path through this context has diverged
    /// (returned, thrown, or exited). Used to detect "all catch branches
    /// return" so that variables assigned only in the try body are
    /// considered definitely assigned after the try/catch.
    pub diverges: bool,

    /// Set once a variable-variable assignment (`${$name} = …`) whose target
    /// name cannot be resolved statically is seen on this path. Such an
    /// assignment may define arbitrarily-named variables, so subsequent reads of
    /// otherwise-unknown variables must not be reported as `UndefinedVariable`
    /// (matching Psalm). Monotonic: once set it propagates through clones/merges.
    pub has_dynamic_var_def: bool,

    /// Pre-converted (line, col_start, line_end, col_end) of the first assignment
    /// to each variable. Used to emit accurate locations for UnusedVariable / UnusedParam.
    pub var_locations: FxHashMap<Name, (u32, u16, u32, u16)>,

    /// Tracks unread write locations per variable.
    /// When a variable is written, its entry is replaced. When the variable
    /// is read as an r-value, its entry is removed (the writes were consumed).
    /// Entries remaining at end-of-scope are dead (last write never read).
    /// Multiple locations arise from branch merges where different paths left
    /// different writes pending (e.g. a pre-loop write and a loop-body write):
    /// a later read consumes ALL of them, since any could be the live value.
    pub last_write_locs: FxHashMap<Name, Vec<(u32, u16, u32, u16)>>,

    /// Dead writes collected during analysis: writes that were overwritten
    /// without being read first. Accumulated via union across branches.
    pub dead_writes: Vec<(Name, u32, u16, u32, u16)>,

    /// Write locations that were consumed by a read on SOME path. A write is
    /// only dead if NO path reads it, so consumption is permanent: merges
    /// union this set and filter pending/dead writes against it, and the
    /// end-of-scope report skips consumed locations. Fixes false positives
    /// like `$x = new stdClass; foreach (...) { $x = $v; } return $x === ...`
    /// where the pre-loop write is overwritten in the body but read on the
    /// loop-never-ran path.
    pub consumed_write_locs: FxHashSet<(Name, (u32, u16, u32, u16))>,

    /// Variables that are foreach iteration values in this scope.
    /// Used to emit UnusedForeachValue instead of UnusedVariable for these names.
    pub foreach_value_var_names: FxHashSet<Name>,

    /// Variables bound by a by-reference foreach (`foreach ($arr as &$val)`).
    /// Writes to these variables always have a side effect (they mutate the
    /// source array through the reference), so dead-write detection must not
    /// flag them as UnusedVariable.
    pub foreach_byref_var_names: FxHashSet<Name>,

    /// Variables bound by a catch clause (`catch (Exception $e)`).
    /// These are declared by the runtime, not by developer assignment, so an
    /// unused catch variable must not produce an UnusedVariable diagnostic.
    pub catch_var_names: FxHashSet<Name>,

    /// Names of template parameters in the current function/method.
    /// Used during type narrowing to correctly handle generic template variables.
    /// Arc-shared — set once at context construction, never mutated during analysis.
    pub template_param_names: Arc<FxHashSet<Name>>,

    /// Names of function/method parameters whose declared types were expanded
    /// from template param bounds (e.g. `@template T of Exception` + `@param T $e`).
    /// Throw analysis uses this to suppress `MissingThrowsDocblock` when a
    /// parameter of generic exception type is re-thrown — the caller owns the
    /// `@throws` declaration, not the generic helper.
    pub template_typed_params: Arc<FxHashSet<Name>>,

    /// FQCNs proven to exist in this branch via a `class_exists()` /
    /// `interface_exists()` / `trait_exists()` guard.  Used to suppress
    /// `UndefinedClass` diagnostics inside guarded branches.
    pub class_exists_guards: ClassExistsGuards,

    /// Constant names proven to exist in this branch via a `defined('NAME')`
    /// guard. Used to suppress `UndefinedConstant` inside guarded branches.
    pub defined_guards: ClassExistsGuards,

    /// Function names proven to exist in this branch via a
    /// `function_exists('fn')` guard. Used to suppress `UndefinedFunction`
    /// inside guarded branches.
    pub function_exists_guards: ClassExistsGuards,

    /// `(expr_key, method_name)` pairs proven to exist via a `method_exists($expr, 'method')`
    /// guard. Used to suppress `UndefinedMethod` inside guarded branches.
    /// `expr_key` is a compact string like `"this->notification"` or `"foo"`.
    pub method_exists_guards: FxHashSet<(Arc<str>, Arc<str>)>,

    /// Narrowed/refined types for instance-property accesses tracked through
    /// the current scope.  Key: `(object_var, prop_name)` e.g. `("this", "id")`.
    /// Set by property assignments and null-guard narrowing; read by
    /// `analyze_property_access` before falling back to the declared type.
    pub prop_refined: Arc<FxHashMap<(Name, Name), Arc<Type>>>,
}

/// Pre-built superglobal initial state, shared across all FlowState instances.
///
/// PHP superglobals are always in scope. Building them fresh per scope costs
/// ~11 Arc allocations + map insertions per function/method. A static snapshot
/// lets each new scope start with a cheap Arc clone; the first local-variable
/// write triggers `Arc::make_mut`, which COW-copies at that point — identical
/// semantics, zero extra allocations on the common path.
fn superglobal_vars() -> &'static Arc<FxHashMap<Name, Arc<Type>>> {
    static VARS: std::sync::OnceLock<Arc<FxHashMap<Name, Arc<Type>>>> = std::sync::OnceLock::new();
    VARS.get_or_init(|| {
        let mixed = Arc::new(mir_types::Type::mixed());
        let mut map = FxHashMap::default();
        for sg in &[
            "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV",
            "GLOBALS", "argv", "argc",
        ] {
            map.insert(Name::from(*sg), Arc::clone(&mixed));
        }
        Arc::new(map)
    })
}

fn superglobal_assigned() -> &'static Arc<FxHashSet<Name>> {
    static ASSIGNED: std::sync::OnceLock<Arc<FxHashSet<Name>>> = std::sync::OnceLock::new();
    ASSIGNED.get_or_init(|| {
        let set: FxHashSet<Name> = [
            "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV",
            "GLOBALS", "argv", "argc",
        ]
        .iter()
        .map(|s| Name::from(*s))
        .collect();
        Arc::new(set)
    })
}

impl FlowState {
    pub fn new() -> Self {
        Self {
            vars: Arc::clone(superglobal_vars()),
            assigned_vars: Arc::clone(superglobal_assigned()),
            possibly_assigned_vars: Arc::new(FxHashSet::default()),
            self_fqcn: None,
            parent_fqcn: None,
            static_fqcn: None,
            fn_return_type: None,
            fn_declared_throws: Arc::from([]),
            inside_loop: false,
            has_dynamic_var_def: false,
            inside_finally: false,
            is_generator: false,
            inside_constructor: false,
            is_in_pure_fn: false,
            inside_static_method: false,
            strict_types: false,
            tainted_vars: FxHashSet::default(),
            read_vars: FxHashSet::default(),
            param_names: Arc::new(FxHashSet::default()),
            byref_param_names: Arc::new(FxHashSet::default()),
            diverges: false,
            var_locations: FxHashMap::default(),
            last_write_locs: FxHashMap::default(),
            dead_writes: Vec::new(),
            consumed_write_locs: FxHashSet::default(),
            foreach_value_var_names: FxHashSet::default(),
            foreach_byref_var_names: FxHashSet::default(),
            catch_var_names: FxHashSet::default(),
            template_param_names: Arc::new(FxHashSet::default()),
            template_typed_params: Arc::new(FxHashSet::default()),
            class_exists_guards: FxHashSet::default(),
            defined_guards: FxHashSet::default(),
            function_exists_guards: FxHashSet::default(),
            method_exists_guards: FxHashSet::default(),
            prop_refined: Arc::new(FxHashMap::default()),
        }
    }

    /// Create a context seeded with the given parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn for_function(
        params: &[mir_codebase::FnParam],
        return_type: Option<Type>,
        declared_throws: Arc<[Arc<str>]>,
        self_fqcn: Option<Arc<str>>,
        parent_fqcn: Option<Arc<str>>,
        static_fqcn: Option<Arc<str>>,
        strict_types: bool,
        is_static: bool,
    ) -> Self {
        Self::for_method(
            params,
            return_type,
            declared_throws,
            self_fqcn,
            parent_fqcn,
            static_fqcn,
            strict_types,
            false,
            is_static,
        )
    }

    /// Like `for_function` but also sets `inside_constructor`.
    #[allow(clippy::too_many_arguments)]
    pub fn for_method(
        params: &[mir_codebase::FnParam],
        return_type: Option<Type>,
        declared_throws: Arc<[Arc<str>]>,
        self_fqcn: Option<Arc<str>>,
        parent_fqcn: Option<Arc<str>>,
        static_fqcn: Option<Arc<str>>,
        strict_types: bool,
        inside_constructor: bool,
        is_static: bool,
    ) -> Self {
        Self::for_method_with_templates(
            params,
            return_type,
            declared_throws,
            self_fqcn,
            parent_fqcn,
            static_fqcn,
            strict_types,
            inside_constructor,
            is_static,
            None,
        )
    }

    /// Like `for_method` but also accepts template parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn for_method_with_templates(
        params: &[mir_codebase::FnParam],
        return_type: Option<Type>,
        declared_throws: Arc<[Arc<str>]>,
        self_fqcn: Option<Arc<str>>,
        parent_fqcn: Option<Arc<str>>,
        static_fqcn: Option<Arc<str>>,
        strict_types: bool,
        inside_constructor: bool,
        is_static: bool,
        template_params: Option<&[mir_codebase::TemplateParam]>,
    ) -> Self {
        let mut ctx = Self::new();
        ctx.fn_return_type = return_type;
        ctx.fn_declared_throws = declared_throws;
        ctx.self_fqcn = self_fqcn.clone();
        ctx.parent_fqcn = parent_fqcn;
        ctx.static_fqcn = static_fqcn;
        ctx.strict_types = strict_types;
        ctx.inside_constructor = inside_constructor;

        // Build local sets — wrap in Arc at the end (set-once, never mutated during analysis).
        let mut template_param_names: FxHashSet<Name> = FxHashSet::default();
        let mut template_typed_params: FxHashSet<Name> = FxHashSet::default();
        let mut param_names: FxHashSet<Name> = FxHashSet::default();
        let mut byref_param_names: FxHashSet<Name> = FxHashSet::default();

        // Build a map of template names to their bounds for parameter type resolution
        let mut template_bounds_map: FxHashMap<Name, Type> = FxHashMap::default();
        if let Some(templates) = template_params {
            for tp in templates {
                let tp_sym = Name::from(tp.name.as_ref());
                template_param_names.insert(tp_sym);
                if let Some(bound) = &tp.bound {
                    template_bounds_map.insert(tp_sym, (**bound).clone());
                }
            }
        }

        for p in params {
            let mut elem_ty =
                p.ty.as_ref()
                    .map(|arc| (**arc).clone())
                    .unwrap_or_else(Type::mixed);

            // Resolve template references to their bounds
            // If the parameter type is a bare unqualified name matching a template parameter,
            // replace it with the template's bound
            let mut is_template_typed = false;
            if elem_ty.types.len() == 1 {
                match &elem_ty.types[0] {
                    mir_types::Atomic::TNamedObject { fqcn, type_params }
                        if type_params.is_empty() && !fqcn.contains('\\') =>
                    {
                        if let Some(bound) = template_bounds_map.get(fqcn) {
                            elem_ty = bound.clone();
                            is_template_typed = true;
                        }
                    }
                    mir_types::Atomic::TTemplateParam { as_type, .. } if !as_type.is_mixed() => {
                        // If the template has a non-mixed bound, use it
                        // Otherwise keep the TTemplateParam to avoid MixedMethodCall errors
                        elem_ty = (**as_type).clone();
                        is_template_typed = true;
                    }
                    _ => {}
                }
            }

            // Variadic params like `Type ...$name` are accessed as `list<Type>` in the body.
            // If the docblock already provides a list/array collection type, don't double-wrap.
            let ty = if p.is_variadic {
                let already_collection = elem_ty.types.iter().any(|a| {
                    matches!(
                        a,
                        mir_types::Atomic::TList { .. }
                            | mir_types::Atomic::TNonEmptyList { .. }
                            | mir_types::Atomic::TArray { .. }
                            | mir_types::Atomic::TNonEmptyArray { .. }
                    )
                });
                if already_collection {
                    elem_ty
                } else {
                    mir_types::Type::single(mir_types::Atomic::TList {
                        value: Box::new(elem_ty),
                    })
                }
            } else {
                elem_ty
            };
            let name = Name::from(p.name.as_ref().trim_start_matches('$'));
            Arc::make_mut(&mut ctx.vars).insert(name, mir_codebase::storage::wrap_var_type(ty));
            Arc::make_mut(&mut ctx.assigned_vars).insert(name);
            param_names.insert(name);
            if is_template_typed {
                template_typed_params.insert(name);
            }
            if p.is_byref {
                byref_param_names.insert(name);
            }
        }

        ctx.inside_static_method = is_static;

        // Inject $this for non-static methods so that $this->method() can be
        // resolved without hitting the mixed-receiver early-return guard.
        if !is_static {
            if let Some(fqcn) = self_fqcn {
                let this_ty = mir_types::Type::single(mir_types::Atomic::TNamedObject {
                    fqcn: mir_types::Name::from(fqcn.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                });
                let this_sym = Name::from("this");
                Arc::make_mut(&mut ctx.vars)
                    .insert(this_sym, mir_codebase::storage::wrap_var_type(this_ty));
                Arc::make_mut(&mut ctx.assigned_vars).insert(this_sym);
            }
        }

        ctx.param_names = Arc::new(param_names);
        ctx.byref_param_names = Arc::new(byref_param_names);
        ctx.template_param_names = Arc::new(template_param_names);
        ctx.template_typed_params = Arc::new(template_typed_params);

        ctx
    }

    /// Get the type of a variable. Returns `mixed` if not found.
    pub fn get_var(&self, name: &str) -> Type {
        let sym = Name::from(name.trim_start_matches('$'));
        self.vars
            .get(&sym)
            .map(|a| (**a).clone())
            .unwrap_or_else(Type::mixed)
    }

    /// Set the type of a variable and mark it as assigned.
    pub fn set_var(&mut self, name: &str, ty: Type) {
        let name = Name::from(name.trim_start_matches('$'));
        Arc::make_mut(&mut self.vars).insert(name, mir_codebase::storage::wrap_var_type(ty));
        Arc::make_mut(&mut self.assigned_vars).insert(name);
    }

    /// Check if a variable is definitely in scope.
    pub fn var_is_defined(&self, name: &str) -> bool {
        let sym = Name::from(name.trim_start_matches('$'));
        self.assigned_vars.contains(&sym)
    }

    /// Check if a variable might be defined (but not certainly).
    pub fn var_possibly_defined(&self, name: &str) -> bool {
        let sym = Name::from(name.trim_start_matches('$'));
        self.assigned_vars.contains(&sym) || self.possibly_assigned_vars.contains(&sym)
    }

    /// Return the narrowed type for an instance-property access, if any.
    /// `obj_var` is the receiver variable name (e.g. `"this"`).
    pub fn get_prop_refined(&self, obj_var: &str, prop: &str) -> Option<&Type> {
        let key = (
            Name::from(obj_var.trim_start_matches('$')),
            Name::from(prop),
        );
        self.prop_refined.get(&key).map(|t| t.as_ref())
    }

    /// Record a narrowed/refined type for an instance property.
    pub fn set_prop_refined(&mut self, obj_var: &str, prop: &str, ty: Type) {
        let key = (
            Name::from(obj_var.trim_start_matches('$')),
            Name::from(prop),
        );
        Arc::make_mut(&mut self.prop_refined).insert(key, mir_codebase::storage::wrap_var_type(ty));
    }

    /// Remove a refined type entry (e.g. after a write that invalidates it).
    #[allow(dead_code)]
    pub fn clear_prop_refined(&mut self, obj_var: &str, prop: &str) {
        let key = (
            Name::from(obj_var.trim_start_matches('$')),
            Name::from(prop),
        );
        Arc::make_mut(&mut self.prop_refined).remove(&key);
    }

    /// Mark a variable as carrying tainted (user-controlled) data.
    pub fn taint_var(&mut self, name: &str) {
        let name = Name::from(name.trim_start_matches('$'));
        self.tainted_vars.insert(name);
    }

    /// Returns true if the variable is known to carry tainted data.
    pub fn is_tainted(&self, name: &str) -> bool {
        let sym = Name::from(name.trim_start_matches('$'));
        self.tainted_vars.contains(&sym)
    }

    /// Record the location of the first assignment to a variable (first-write-wins)
    /// and update the dead-write tracking for this variable.
    pub fn record_var_location(
        &mut self,
        name: &str,
        line: u32,
        col_start: u16,
        line_end: u32,
        col_end: u16,
    ) {
        let sym = Name::from(name.trim_start_matches('$'));
        self.var_locations
            .entry(sym)
            .or_insert((line, col_start, line_end, col_end));
        self.record_write(name, line, col_start, line_end, col_end);
    }

    /// Record a write to a variable for dead-write tracking.
    ///
    /// If the variable had an unread write since the last read, that previous
    /// write is collected as a dead write. The new write location becomes the
    /// current pending write.
    ///
    /// Call this alongside `record_var_location` at every PHP-level assignment
    /// (but NOT for type-narrowing `set_var` calls in the narrowing engine).
    pub fn record_write(
        &mut self,
        name: &str,
        line: u32,
        col_start: u16,
        line_end: u32,
        col_end: u16,
    ) {
        let sym = Name::from(name.trim_start_matches('$'));
        if let Some(old_locs) = self.last_write_locs.get(&sym) {
            // Previous write(s) overwritten without being read → dead writes.
            for old_loc in old_locs.clone() {
                self.dead_writes
                    .push((sym, old_loc.0, old_loc.1, old_loc.2, old_loc.3));
            }
        }
        self.last_write_locs
            .insert(sym, vec![(line, col_start, line_end, col_end)]);
    }

    /// Mark a variable as consumed by an r-value read.
    ///
    /// This clears the pending write entry so the write is no longer considered
    /// dead. Call this whenever a variable is used as an expression value.
    pub fn mark_consumed(&mut self, name: &str) {
        let sym = Name::from(name.trim_start_matches('$'));
        if let Some(locs) = self.last_write_locs.remove(&sym) {
            // Consumption is permanent across branch merges: see
            // `consumed_write_locs`.
            for loc in locs {
                self.consumed_write_locs.insert((sym, loc));
            }
        }
    }

    /// Propagate reads observed in a branch-scoped context (ternary arm,
    /// match arm, closure body) back into this context: variables read there
    /// count as read here, and write locations they consumed stay consumed.
    pub fn absorb_branch_reads(&mut self, branch: &FlowState) {
        for name in branch.read_vars.iter() {
            self.read_vars.insert(*name);
        }
        for key in branch.consumed_write_locs.iter() {
            self.consumed_write_locs.insert(*key);
        }
        let consumed = &self.consumed_write_locs;
        self.last_write_locs.retain(|name, locs| {
            locs.retain(|loc| !consumed.contains(&(*name, *loc)));
            !locs.is_empty()
        });
    }

    /// Remove a variable from the context (after `unset`).
    pub fn unset_var(&mut self, name: &str) {
        let sym = Name::from(name.trim_start_matches('$'));
        Arc::make_mut(&mut self.vars).remove(&sym);
        Arc::make_mut(&mut self.assigned_vars).remove(&sym);
        Arc::make_mut(&mut self.possibly_assigned_vars).remove(&sym);
    }

    /// Clone this context to analyze a conditional branch (`if`, `elseif`,
    /// `else`, `case`, ternary arm, …). The returned context can be mutated
    /// independently and later reconciled via [`Self::merge_branches`].
    pub fn branch(&self) -> FlowState {
        crate::metrics::record_flow_branch(
            self.read_vars.len(),
            self.var_locations.len(),
            self.last_write_locs.len(),
        );
        self.clone()
    }

    /// Merge two branch contexts at a join point (e.g. end of if/else).
    ///
    /// - vars present in both: merged union of types
    /// - vars present in only one branch: marked `possibly_undefined`
    /// - pre-existing vars from before the branch: preserved
    pub fn merge_branches(
        pre: &FlowState,
        if_ctx: FlowState,
        else_ctx: Option<FlowState>,
    ) -> FlowState {
        let else_ctx = else_ctx.unwrap_or_else(|| pre.clone());

        // A dynamic variable definition seen on any incoming path is sticky: it
        // must survive every merge so later undefined-variable checks stay
        // suppressed (see `FlowState::has_dynamic_var_def`).
        let dynamic_var_def =
            pre.has_dynamic_var_def || if_ctx.has_dynamic_var_def || else_ctx.has_dynamic_var_def;

        // If the then-branch always diverges, the code after the if runs only
        // in the else-branch — use that as the result directly.
        if if_ctx.diverges && !else_ctx.diverges {
            let mut result = else_ctx;
            result.diverges = false;
            // Variables read in the diverging branch still count as used.
            for name in if_ctx.read_vars.iter() {
                result.read_vars.insert(*name);
            }
            result
                .consumed_write_locs
                .extend(if_ctx.consumed_write_locs.iter().copied());
            // Remove pending writes that were consumed in the diverging branch.
            // Use consumed_write_locs rather than read_vars to avoid incorrectly
            // removing writes that appear in read_vars only due to loop-iteration
            // accumulation (not because they were actually consumed this iteration).
            {
                let consumed = result.consumed_write_locs.clone();
                result.last_write_locs.retain(|name, locs| {
                    locs.retain(|loc| !consumed.contains(&(*name, *loc)));
                    !locs.is_empty()
                });
            }
            // Dead writes from the diverging branch are still dead.
            extend_dead_writes_dedup(&mut result.dead_writes, if_ctx.dead_writes);
            for name in if_ctx.foreach_value_var_names.iter() {
                result.foreach_value_var_names.insert(*name);
            }
            for name in if_ctx.foreach_byref_var_names.iter() {
                result.foreach_byref_var_names.insert(*name);
            }
            for name in if_ctx.catch_var_names.iter() {
                result.catch_var_names.insert(*name);
            }
            result.has_dynamic_var_def = dynamic_var_def;
            return result;
        }
        // If the else-branch always diverges, code after the if runs only
        // in the then-branch.
        if else_ctx.diverges && !if_ctx.diverges {
            let mut result = if_ctx;
            result.diverges = false;
            // Variables read in the diverging branch still count as used.
            for name in else_ctx.read_vars.iter() {
                result.read_vars.insert(*name);
            }
            result
                .consumed_write_locs
                .extend(else_ctx.consumed_write_locs.iter().copied());
            // Remove pending writes consumed in the diverging branch.
            {
                let consumed = result.consumed_write_locs.clone();
                result.last_write_locs.retain(|name, locs| {
                    locs.retain(|loc| !consumed.contains(&(*name, *loc)));
                    !locs.is_empty()
                });
            }
            extend_dead_writes_dedup(&mut result.dead_writes, else_ctx.dead_writes);
            for name in else_ctx.foreach_value_var_names.iter() {
                result.foreach_value_var_names.insert(*name);
            }
            for name in else_ctx.foreach_byref_var_names.iter() {
                result.foreach_byref_var_names.insert(*name);
            }
            for name in else_ctx.catch_var_names.iter() {
                result.catch_var_names.insert(*name);
            }
            result.has_dynamic_var_def = dynamic_var_def;
            return result;
        }
        // If both diverge, the code after the if is unreachable.
        if if_ctx.diverges && else_ctx.diverges {
            let mut result = pre.clone();
            result.diverges = true;
            // Variables read in either diverging branch still count as used.
            for name in if_ctx.read_vars.iter().chain(else_ctx.read_vars.iter()) {
                result.read_vars.insert(*name);
            }
            result
                .consumed_write_locs
                .extend(if_ctx.consumed_write_locs.iter().copied());
            result
                .consumed_write_locs
                .extend(else_ctx.consumed_write_locs.iter().copied());
            // `result` is `pre.clone()`; both branches already contain pre's
            // dead writes, so rebuild deduped rather than concatenating.
            result.dead_writes.clear();
            extend_dead_writes_dedup(&mut result.dead_writes, if_ctx.dead_writes);
            extend_dead_writes_dedup(&mut result.dead_writes, else_ctx.dead_writes);
            for name in if_ctx
                .foreach_value_var_names
                .iter()
                .chain(else_ctx.foreach_value_var_names.iter())
            {
                result.foreach_value_var_names.insert(*name);
            }
            for name in if_ctx
                .foreach_byref_var_names
                .iter()
                .chain(else_ctx.foreach_byref_var_names.iter())
            {
                result.foreach_byref_var_names.insert(*name);
            }
            for name in if_ctx
                .catch_var_names
                .iter()
                .chain(else_ctx.catch_var_names.iter())
            {
                result.catch_var_names.insert(*name);
            }
            result.has_dynamic_var_def = dynamic_var_def;
            return result;
        }

        let mut result = pre.clone();

        // Collect all variable names from both branch contexts
        let all_names: FxHashSet<Name> = if_ctx
            .vars
            .keys()
            .chain(else_ctx.vars.keys())
            .copied()
            .collect();

        {
            let result_vars = Arc::make_mut(&mut result.vars);
            let result_assigned = Arc::make_mut(&mut result.assigned_vars);
            let result_possibly = Arc::make_mut(&mut result.possibly_assigned_vars);

            for name in all_names {
                let in_if = if_ctx.assigned_vars.contains(&name);
                let in_else = else_ctx.assigned_vars.contains(&name);
                let in_pre = pre.assigned_vars.contains(&name);

                let ty_if = if_ctx.vars.get(&name);
                let ty_else = else_ctx.vars.get(&name);

                match (ty_if, ty_else) {
                    (Some(a), Some(b)) => {
                        let merged = if Arc::ptr_eq(a, b) {
                            a.clone()
                        } else {
                            let mut m = (**a).clone();
                            m.merge_with(b);
                            mir_codebase::storage::wrap_var_type(m)
                        };
                        result_vars.insert(name, merged);
                        if in_if && in_else {
                            result_assigned.insert(name);
                        } else {
                            result_possibly.insert(name);
                        }
                    }
                    (Some(a), None) => {
                        if in_pre {
                            let pre_arc = pre.vars.get(&name);
                            let merged = match pre_arc {
                                Some(pt) if Arc::ptr_eq(a, pt) => a.clone(),
                                Some(pt) => {
                                    let mut m = (**a).clone();
                                    m.merge_with(pt);
                                    mir_codebase::storage::wrap_var_type(m)
                                }
                                None => {
                                    let mut m = (**a).clone();
                                    m.merge_with(&Type::mixed());
                                    mir_codebase::storage::wrap_var_type(m)
                                }
                            };
                            result_vars.insert(name, merged);
                            result_assigned.insert(name);
                        } else {
                            let ty = mir_codebase::storage::wrap_var_type(
                                (**a).clone().possibly_undefined(),
                            );
                            result_vars.insert(name, ty);
                            result_possibly.insert(name);
                        }
                    }
                    (None, Some(b)) => {
                        if in_pre {
                            let pre_arc = pre.vars.get(&name);
                            let merged = match pre_arc {
                                Some(pt) if Arc::ptr_eq(b, pt) => b.clone(),
                                Some(pt) => {
                                    let mut m = (**pt).clone();
                                    m.merge_with(b);
                                    mir_codebase::storage::wrap_var_type(m)
                                }
                                None => {
                                    let mut m = Type::mixed();
                                    m.merge_with(b);
                                    mir_codebase::storage::wrap_var_type(m)
                                }
                            };
                            result_vars.insert(name, merged);
                            result_assigned.insert(name);
                        } else {
                            let ty = mir_codebase::storage::wrap_var_type(
                                (**b).clone().possibly_undefined(),
                            );
                            result_vars.insert(name, ty);
                            result_possibly.insert(name);
                        }
                    }
                    (None, None) => {}
                }
            }
        }

        // Class-exists guards: intersection — a guard survives the merge only if
        // both branches have it, meaning the class is guaranteed to exist on every
        // path.  In the common case (only the then-branch has the guard) the
        // intersection is empty, which is correct: after the if/else the guard no
        // longer applies.
        result.class_exists_guards = if_ctx
            .class_exists_guards
            .intersection(&else_ctx.class_exists_guards)
            .cloned()
            .collect();
        result.defined_guards = if_ctx
            .defined_guards
            .intersection(&else_ctx.defined_guards)
            .cloned()
            .collect();
        result.function_exists_guards = if_ctx
            .function_exists_guards
            .intersection(&else_ctx.function_exists_guards)
            .cloned()
            .collect();
        result.method_exists_guards = if_ctx
            .method_exists_guards
            .intersection(&else_ctx.method_exists_guards)
            .cloned()
            .collect();

        // Property refinements: keep only keys present in BOTH branches with the
        // union of their types.  A refinement present in only one branch cannot
        // be relied upon after the merge (the other path didn't narrow it).
        {
            let mut merged_props: FxHashMap<(Name, Name), Arc<Type>> = FxHashMap::default();
            for (key, if_ty) in if_ctx.prop_refined.iter() {
                if let Some(else_ty) = else_ctx.prop_refined.get(key) {
                    let merged = if Arc::ptr_eq(if_ty, else_ty) {
                        if_ty.clone()
                    } else {
                        let mut m = (**if_ty).clone();
                        m.merge_with(else_ty);
                        mir_codebase::storage::wrap_var_type(m)
                    };
                    merged_props.insert(*key, merged);
                }
            }
            result.prop_refined = Arc::new(merged_props);
        }

        // Taint: conservative union — if either branch taints a var, it stays tainted
        for name in if_ctx
            .tainted_vars
            .iter()
            .chain(else_ctx.tainted_vars.iter())
        {
            result.tainted_vars.insert(*name);
        }

        // Read vars: union — if either branch reads a var, it counts as read
        for name in if_ctx.read_vars.iter().chain(else_ctx.read_vars.iter()) {
            result.read_vars.insert(*name);
        }

        result.has_dynamic_var_def = dynamic_var_def;

        // Foreach value var names: union — if either branch marks a var as a foreach value, keep it
        for name in if_ctx
            .foreach_value_var_names
            .iter()
            .chain(else_ctx.foreach_value_var_names.iter())
        {
            result.foreach_value_var_names.insert(*name);
        }

        // Foreach by-ref var names: union — if either branch marks a var as by-ref, keep it
        for name in if_ctx
            .foreach_byref_var_names
            .iter()
            .chain(else_ctx.foreach_byref_var_names.iter())
        {
            result.foreach_byref_var_names.insert(*name);
        }

        // Catch var names: union — catch variables from any branch must not be reported
        for name in if_ctx
            .catch_var_names
            .iter()
            .chain(else_ctx.catch_var_names.iter())
        {
            result.catch_var_names.insert(*name);
        }

        // Var locations: keep the earliest known span for each variable
        for (name, loc) in if_ctx
            .var_locations
            .iter()
            .chain(else_ctx.var_locations.iter())
        {
            result.var_locations.entry(*name).or_insert(*loc);
        }

        // Dead writes: union of both branches, deduplicated. `result` is
        // `pre.clone()` and both branches descend from `pre`, so they already
        // contain pre's dead writes — rebuild from the branches rather than
        // concatenating onto pre's copy (which would grow the Vec ~3× per merge
        // → exponential under nested-loop fixpoint analysis; see
        // `extend_dead_writes_dedup`).
        result.dead_writes.clear();
        extend_dead_writes_dedup(&mut result.dead_writes, if_ctx.dead_writes);
        extend_dead_writes_dedup(&mut result.dead_writes, else_ctx.dead_writes);

        // Consumed write locations: union — a write read on ANY path is used.
        result.consumed_write_locs = if_ctx.consumed_write_locs;
        result
            .consumed_write_locs
            .extend(else_ctx.consumed_write_locs.iter().copied());

        // Last write locs: union from both branches, plus pre_ctx variables that
        // are still pending in BOTH branches (meaning neither branch nor the
        // condition consumed them). Variables present in pre_ctx but absent from
        // both branches were consumed on all paths (e.g. read in condition) and
        // must not be re-added. Entries whose location was consumed on either
        // path are filtered — that write is used.
        result.last_write_locs = FxHashMap::default();
        {
            let consumed = &result.consumed_write_locs;
            let add = |map: &mut FxHashMap<Name, Vec<(u32, u16, u32, u16)>>,
                       name: &Name,
                       loc: &(u32, u16, u32, u16)| {
                if !consumed.contains(&(*name, *loc)) {
                    let entry = map.entry(*name).or_default();
                    if !entry.contains(loc) {
                        entry.push(*loc);
                    }
                }
            };
            // Branch contexts inherit pre's pending entries, so unioning the two
            // branch maps already covers pre-existing writes: a pre write still
            // pending in either branch survives; one overwritten in BOTH
            // branches (dead on every path) is gone from both maps and stays
            // out. No separate pre re-add is needed — re-adding by name would
            // resurrect pre locations that both branches overwrote.
            for (name, locs) in if_ctx
                .last_write_locs
                .iter()
                .chain(else_ctx.last_write_locs.iter())
            {
                for loc in locs.iter() {
                    add(&mut result.last_write_locs, name, loc);
                }
            }
        }

        // After merging branches, the merged context does not diverge
        // (at least one path through the merge reaches the next statement).
        result.diverges = false;

        result
    }
}

impl Default for FlowState {
    fn default() -> Self {
        Self::new()
    }
}

/// True when the method/closure currently being analyzed belongs to a trait
/// (i.e. `self_fqcn` names a trait). Trait bodies are analyzed standalone, so
/// `self`/`static`/`parent`/`$this` resolve against the trait rather than the
/// (unknown) using class — callers use this to suppress class-resolution
/// false positives that only hold for the trait in isolation.
pub(crate) fn self_is_trait(db: &dyn crate::db::MirDatabase, ctx: &FlowState) -> bool {
    ctx.self_fqcn
        .as_deref()
        .is_some_and(|f| crate::db::class_kind(db, f).is_some_and(|k| k.is_trait))
}
