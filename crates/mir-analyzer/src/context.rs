/// Analysis context — carries type state through statement/expression analysis.
use rustc_hash::{FxHashMap, FxHashSet};
use std::sync::Arc;

use mir_types::{Symbol, Union};

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Context {
    /// Types of variables at this point in execution.
    pub vars: im::HashMap<Symbol, Union>,

    /// Variables that are definitely assigned at this point.
    pub assigned_vars: FxHashSet<Symbol>,

    /// Variables that *might* be assigned (e.g. only in one if branch).
    pub possibly_assigned_vars: FxHashSet<Symbol>,

    /// The class in whose body we are analysing (`self`).
    pub self_fqcn: Option<Arc<str>>,

    /// The parent class (`parent`).
    pub parent_fqcn: Option<Arc<str>>,

    /// Late-static-binding class (`static`).
    pub static_fqcn: Option<Arc<str>>,

    /// Declared return type for the current function/method.
    pub fn_return_type: Option<Union>,

    /// Declared exception types for the current function/method (@throws).
    pub fn_declared_throws: Arc<[Arc<str>]>,

    /// Whether we are currently inside a loop.
    pub inside_loop: bool,

    /// Whether we are currently inside a finally block.
    pub inside_finally: bool,

    /// Whether we are inside a constructor.
    pub inside_constructor: bool,

    /// Whether `strict_types=1` is declared for this file.
    pub strict_types: bool,

    /// Variables that carry tainted (user-controlled) values at this point.
    /// Used by taint analysis (M19).
    pub tainted_vars: FxHashSet<Symbol>,

    /// Variables that have been read at least once in this scope.
    /// Used by UnusedParam detection (M18).
    pub read_vars: FxHashSet<Symbol>,

    /// Names of function/method parameters in this scope (stripped of `$`).
    /// Used to exclude parameters from UnusedVariable detection.
    pub param_names: FxHashSet<Symbol>,

    /// Names of by-reference parameters in this scope (stripped of `$`).
    /// Assigning to these is externally observable, so it counts as usage.
    pub byref_param_names: FxHashSet<Symbol>,

    /// Whether every execution path through this context has diverged
    /// (returned, thrown, or exited). Used to detect "all catch branches
    /// return" so that variables assigned only in the try body are
    /// considered definitely assigned after the try/catch.
    pub diverges: bool,

    /// Pre-converted (line, col_start, line_end, col_end) of the first assignment
    /// to each variable. Used to emit accurate locations for UnusedVariable / UnusedParam.
    pub var_locations: FxHashMap<Symbol, (u32, u16, u32, u16)>,

    /// Names of template parameters in the current function/method.
    /// Used during type narrowing to correctly handle generic template variables.
    pub template_param_names: FxHashSet<Symbol>,
}

impl Context {
    pub fn new() -> Self {
        let mut ctx = Self {
            vars: im::HashMap::new(),
            assigned_vars: FxHashSet::default(),
            possibly_assigned_vars: FxHashSet::default(),
            self_fqcn: None,
            parent_fqcn: None,
            static_fqcn: None,
            fn_return_type: None,
            fn_declared_throws: Arc::from([]),
            inside_loop: false,
            inside_finally: false,
            inside_constructor: false,
            strict_types: false,
            tainted_vars: FxHashSet::default(),
            read_vars: FxHashSet::default(),
            param_names: FxHashSet::default(),
            byref_param_names: FxHashSet::default(),
            diverges: false,
            var_locations: FxHashMap::default(),
            template_param_names: FxHashSet::default(),
        };
        // PHP superglobals — always in scope in any context
        for sg in &[
            "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV",
            "GLOBALS",
        ] {
            let sym = Symbol::from(*sg);
            ctx.vars.insert(sym, mir_types::Union::mixed());
            ctx.assigned_vars.insert(sym);
        }
        ctx
    }

    /// Create a context seeded with the given parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn for_function(
        params: &[mir_codebase::FnParam],
        return_type: Option<Union>,
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
        return_type: Option<Union>,
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
        return_type: Option<Union>,
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

        // Build a map of template names to their bounds for parameter type resolution
        let mut template_bounds_map: FxHashMap<Symbol, Union> = FxHashMap::default();
        if let Some(templates) = template_params {
            for tp in templates {
                let tp_sym = Symbol::from(tp.name.as_ref());
                ctx.template_param_names.insert(tp_sym);
                if let Some(bound) = &tp.bound {
                    template_bounds_map.insert(tp_sym, bound.clone());
                }
            }
        }

        for p in params {
            let mut elem_ty =
                p.ty.as_ref()
                    .map(|arc| (**arc).clone())
                    .unwrap_or_else(Union::mixed);

            // Resolve template references to their bounds
            // If the parameter type is a bare unqualified name matching a template parameter,
            // replace it with the template's bound
            if elem_ty.types.len() == 1 {
                match &elem_ty.types[0] {
                    mir_types::Atomic::TNamedObject { fqcn, type_params }
                        if type_params.is_empty() && !fqcn.contains('\\') =>
                    {
                        if let Some(bound) = template_bounds_map.get(fqcn) {
                            elem_ty = bound.clone();
                        }
                    }
                    mir_types::Atomic::TTemplateParam { as_type, .. } if !as_type.is_mixed() => {
                        // If the template has a non-mixed bound, use it
                        // Otherwise keep the TTemplateParam to avoid MixedMethodCall errors
                        elem_ty = (**as_type).clone();
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
                    mir_types::Union::single(mir_types::Atomic::TList {
                        value: Box::new(elem_ty),
                    })
                }
            } else {
                elem_ty
            };
            let name = Symbol::from(p.name.as_ref().trim_start_matches('$'));
            ctx.vars.insert(name, ty);
            ctx.assigned_vars.insert(name);
            ctx.param_names.insert(name);
            if p.is_byref {
                ctx.byref_param_names.insert(name);
            }
        }

        // Inject $this for non-static methods so that $this->method() can be
        // resolved without hitting the mixed-receiver early-return guard.
        if !is_static {
            if let Some(fqcn) = self_fqcn {
                let this_ty = mir_types::Union::single(mir_types::Atomic::TNamedObject {
                    fqcn: mir_types::Symbol::from(fqcn.as_ref()),
                    type_params: mir_types::union::empty_type_params(),
                });
                let this_sym = Symbol::from("this");
                ctx.vars.insert(this_sym, this_ty);
                ctx.assigned_vars.insert(this_sym);
            }
        }

        ctx
    }

    /// Get the type of a variable. Returns `mixed` if not found.
    pub fn get_var(&self, name: &str) -> Union {
        let sym = Symbol::from(name.trim_start_matches('$'));
        self.vars.get(&sym).cloned().unwrap_or_else(Union::mixed)
    }

    /// Set the type of a variable and mark it as assigned.
    pub fn set_var(&mut self, name: impl Into<String>, ty: Union) {
        let name: String = name.into();
        let name = Symbol::from(name.trim_start_matches('$'));
        self.vars.insert(name, ty);
        self.assigned_vars.insert(name);
    }

    /// Check if a variable is definitely in scope.
    pub fn var_is_defined(&self, name: &str) -> bool {
        let sym = Symbol::from(name.trim_start_matches('$'));
        self.assigned_vars.contains(&sym)
    }

    /// Check if a variable might be defined (but not certainly).
    pub fn var_possibly_defined(&self, name: &str) -> bool {
        let sym = Symbol::from(name.trim_start_matches('$'));
        self.assigned_vars.contains(&sym) || self.possibly_assigned_vars.contains(&sym)
    }

    /// Mark a variable as carrying tainted (user-controlled) data.
    pub fn taint_var(&mut self, name: &str) {
        let name = Symbol::from(name.trim_start_matches('$'));
        self.tainted_vars.insert(name);
    }

    /// Returns true if the variable is known to carry tainted data.
    pub fn is_tainted(&self, name: &str) -> bool {
        let sym = Symbol::from(name.trim_start_matches('$'));
        self.tainted_vars.contains(&sym)
    }

    /// Record the location of the first assignment to a variable (first-write-wins).
    pub fn record_var_location(
        &mut self,
        name: &str,
        line: u32,
        col_start: u16,
        line_end: u32,
        col_end: u16,
    ) {
        let name = Symbol::from(name.trim_start_matches('$'));
        self.var_locations
            .entry(name)
            .or_insert((line, col_start, line_end, col_end));
    }

    /// Remove a variable from the context (after `unset`).
    pub fn unset_var(&mut self, name: &str) {
        let sym = Symbol::from(name.trim_start_matches('$'));
        self.vars.remove(&sym);
        self.assigned_vars.remove(&sym);
        self.possibly_assigned_vars.remove(&sym);
    }

    /// Fork this context for a branch (e.g. the `if` branch).
    pub fn fork(&self) -> Context {
        self.clone()
    }

    /// Merge two branch contexts at a join point (e.g. end of if/else).
    ///
    /// - vars present in both: merged union of types
    /// - vars present in only one branch: marked `possibly_undefined`
    /// - pre-existing vars from before the branch: preserved
    pub fn merge_branches(pre: &Context, if_ctx: Context, else_ctx: Option<Context>) -> Context {
        let else_ctx = else_ctx.unwrap_or_else(|| pre.clone());

        // If the then-branch always diverges, the code after the if runs only
        // in the else-branch — use that as the result directly.
        if if_ctx.diverges && !else_ctx.diverges {
            let mut result = else_ctx;
            result.diverges = false;
            return result;
        }
        // If the else-branch always diverges, code after the if runs only
        // in the then-branch.
        if else_ctx.diverges && !if_ctx.diverges {
            let mut result = if_ctx;
            result.diverges = false;
            return result;
        }
        // If both diverge, the code after the if is unreachable.
        if if_ctx.diverges && else_ctx.diverges {
            let mut result = pre.clone();
            result.diverges = true;
            return result;
        }

        let mut result = pre.clone();

        // Collect all variable names from both branch contexts
        let all_names: FxHashSet<Symbol> = if_ctx
            .vars
            .keys()
            .chain(else_ctx.vars.keys())
            .copied()
            .collect();

        for name in all_names {
            let in_if = if_ctx.assigned_vars.contains(&name);
            let in_else = else_ctx.assigned_vars.contains(&name);
            let in_pre = pre.assigned_vars.contains(&name);

            let ty_if = if_ctx.vars.get(&name);
            let ty_else = else_ctx.vars.get(&name);

            match (ty_if, ty_else) {
                (Some(a), Some(b)) => {
                    let mut merged = a.clone();
                    merged.merge_with(b);
                    result.vars.insert(name, merged);
                    if in_if && in_else {
                        result.assigned_vars.insert(name);
                    } else {
                        result.possibly_assigned_vars.insert(name);
                    }
                }
                (Some(a), None) => {
                    if in_pre {
                        // var existed before: merge with pre type
                        let pre_ty = pre.vars.get(&name).cloned().unwrap_or_else(Union::mixed);
                        let mut merged = a.clone();
                        merged.merge_with(&pre_ty);
                        result.vars.insert(name, merged);
                        result.assigned_vars.insert(name);
                    } else {
                        // only assigned in if branch
                        let ty = a.clone().possibly_undefined();
                        result.vars.insert(name, ty);
                        result.possibly_assigned_vars.insert(name);
                    }
                }
                (None, Some(b)) => {
                    if in_pre {
                        let pre_ty = pre.vars.get(&name).cloned().unwrap_or_else(Union::mixed);
                        let mut merged = pre_ty;
                        merged.merge_with(b);
                        result.vars.insert(name, merged);
                        result.assigned_vars.insert(name);
                    } else {
                        let ty = b.clone().possibly_undefined();
                        result.vars.insert(name, ty);
                        result.possibly_assigned_vars.insert(name);
                    }
                }
                (None, None) => {}
            }
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

        // Var locations: keep the earliest known span for each variable
        for (name, loc) in if_ctx
            .var_locations
            .iter()
            .chain(else_ctx.var_locations.iter())
        {
            result.var_locations.entry(*name).or_insert(*loc);
        }

        // After merging branches, the merged context does not diverge
        // (at least one path through the merge reaches the next statement).
        result.diverges = false;

        result
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}
