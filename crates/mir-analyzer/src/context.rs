/// Analysis context — carries type state through statement/expression analysis.
use std::collections::HashSet;
use std::sync::Arc;

use indexmap::IndexMap;
use mir_types::Union;

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Context {
    /// Types of variables at this point in execution.
    pub vars: IndexMap<String, Union>,

    /// Variables that are definitely assigned at this point.
    pub assigned_vars: HashSet<String>,

    /// Variables that *might* be assigned (e.g. only in one if branch).
    pub possibly_assigned_vars: HashSet<String>,

    /// The class in whose body we are analysing (`self`).
    pub self_fqcn: Option<Arc<str>>,

    /// The parent class (`parent`).
    pub parent_fqcn: Option<Arc<str>>,

    /// Late-static-binding class (`static`).
    pub static_fqcn: Option<Arc<str>>,

    /// Declared return type for the current function/method.
    pub fn_return_type: Option<Union>,

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
    pub tainted_vars: HashSet<String>,

    /// Variables that have been read at least once in this scope.
    /// Used by UnusedParam detection (M18).
    pub read_vars: HashSet<String>,

    /// Names of function/method parameters in this scope (stripped of `$`).
    /// Used to exclude parameters from UnusedVariable detection.
    pub param_names: HashSet<String>,

    /// Whether every execution path through this context has diverged
    /// (returned, thrown, or exited). Used to detect "all catch branches
    /// return" so that variables assigned only in the try body are
    /// considered definitely assigned after the try/catch.
    pub diverges: bool,
}

impl Context {
    pub fn new() -> Self {
        let mut ctx = Self {
            vars: IndexMap::new(),
            assigned_vars: HashSet::new(),
            possibly_assigned_vars: HashSet::new(),
            self_fqcn: None,
            parent_fqcn: None,
            static_fqcn: None,
            fn_return_type: None,
            inside_loop: false,
            inside_finally: false,
            inside_constructor: false,
            strict_types: false,
            tainted_vars: HashSet::new(),
            read_vars: HashSet::new(),
            param_names: HashSet::new(),
            diverges: false,
        };
        // PHP superglobals — always in scope in any context
        for sg in &[
            "_SERVER", "_GET", "_POST", "_REQUEST", "_SESSION", "_COOKIE", "_FILES", "_ENV",
            "GLOBALS",
        ] {
            ctx.vars.insert(sg.to_string(), mir_types::Union::mixed());
            ctx.assigned_vars.insert(sg.to_string());
        }
        ctx
    }

    /// Create a context seeded with the given parameters.
    pub fn for_function(
        params: &[mir_codebase::FnParam],
        return_type: Option<Union>,
        self_fqcn: Option<Arc<str>>,
        parent_fqcn: Option<Arc<str>>,
        static_fqcn: Option<Arc<str>>,
        strict_types: bool,
        is_static: bool,
    ) -> Self {
        Self::for_method(
            params,
            return_type,
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
        self_fqcn: Option<Arc<str>>,
        parent_fqcn: Option<Arc<str>>,
        static_fqcn: Option<Arc<str>>,
        strict_types: bool,
        inside_constructor: bool,
        is_static: bool,
    ) -> Self {
        let mut ctx = Self::new();
        ctx.fn_return_type = return_type;
        ctx.self_fqcn = self_fqcn.clone();
        ctx.parent_fqcn = parent_fqcn;
        ctx.static_fqcn = static_fqcn;
        ctx.strict_types = strict_types;
        ctx.inside_constructor = inside_constructor;

        for p in params {
            let elem_ty = p.ty.clone().unwrap_or_else(Union::mixed);
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
            let name = p.name.as_ref().trim_start_matches('$').to_string();
            ctx.vars.insert(name.clone(), ty);
            ctx.assigned_vars.insert(name.clone());
            ctx.param_names.insert(name);
        }

        // Inject $this for non-static methods so that $this->method() can be
        // resolved without hitting the mixed-receiver early-return guard.
        if !is_static {
            if let Some(fqcn) = self_fqcn {
                let this_ty = mir_types::Union::single(mir_types::Atomic::TNamedObject {
                    fqcn,
                    type_params: vec![],
                });
                ctx.vars.insert("this".to_string(), this_ty);
                ctx.assigned_vars.insert("this".to_string());
            }
        }

        ctx
    }

    /// Get the type of a variable. Returns `mixed` if not found.
    pub fn get_var(&self, name: &str) -> Union {
        let name = name.trim_start_matches('$');
        self.vars.get(name).cloned().unwrap_or_else(Union::mixed)
    }

    /// Set the type of a variable and mark it as assigned.
    pub fn set_var(&mut self, name: impl Into<String>, ty: Union) {
        let name: String = name.into();
        let name = name.trim_start_matches('$').to_string();
        self.vars.insert(name.clone(), ty);
        self.assigned_vars.insert(name);
    }

    /// Check if a variable is definitely in scope.
    pub fn var_is_defined(&self, name: &str) -> bool {
        let name = name.trim_start_matches('$');
        self.assigned_vars.contains(name)
    }

    /// Check if a variable might be defined (but not certainly).
    pub fn var_possibly_defined(&self, name: &str) -> bool {
        let name = name.trim_start_matches('$');
        self.assigned_vars.contains(name) || self.possibly_assigned_vars.contains(name)
    }

    /// Mark a variable as carrying tainted (user-controlled) data.
    pub fn taint_var(&mut self, name: &str) {
        let name = name.trim_start_matches('$').to_string();
        self.tainted_vars.insert(name);
    }

    /// Returns true if the variable is known to carry tainted data.
    pub fn is_tainted(&self, name: &str) -> bool {
        let name = name.trim_start_matches('$');
        self.tainted_vars.contains(name)
    }

    /// Remove a variable from the context (after `unset`).
    pub fn unset_var(&mut self, name: &str) {
        let name = name.trim_start_matches('$');
        self.vars.shift_remove(name);
        self.assigned_vars.remove(name);
        self.possibly_assigned_vars.remove(name);
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
        let all_names: HashSet<&String> = if_ctx.vars.keys().chain(else_ctx.vars.keys()).collect();

        for name in all_names {
            let in_if = if_ctx.assigned_vars.contains(name);
            let in_else = else_ctx.assigned_vars.contains(name);
            let in_pre = pre.assigned_vars.contains(name);

            let ty_if = if_ctx.vars.get(name);
            let ty_else = else_ctx.vars.get(name);

            match (ty_if, ty_else) {
                (Some(a), Some(b)) => {
                    let merged = Union::merge(a, b);
                    result.vars.insert(name.clone(), merged);
                    if in_if && in_else {
                        result.assigned_vars.insert(name.clone());
                    } else {
                        result.possibly_assigned_vars.insert(name.clone());
                    }
                }
                (Some(a), None) => {
                    if in_pre {
                        // var existed before: merge with pre type
                        let pre_ty = pre.vars.get(name).cloned().unwrap_or_else(Union::mixed);
                        let merged = Union::merge(a, &pre_ty);
                        result.vars.insert(name.clone(), merged);
                        result.assigned_vars.insert(name.clone());
                    } else {
                        // only assigned in if branch
                        let ty = a.clone().possibly_undefined();
                        result.vars.insert(name.clone(), ty);
                        result.possibly_assigned_vars.insert(name.clone());
                    }
                }
                (None, Some(b)) => {
                    if in_pre {
                        let pre_ty = pre.vars.get(name).cloned().unwrap_or_else(Union::mixed);
                        let merged = Union::merge(&pre_ty, b);
                        result.vars.insert(name.clone(), merged);
                        result.assigned_vars.insert(name.clone());
                    } else {
                        let ty = b.clone().possibly_undefined();
                        result.vars.insert(name.clone(), ty);
                        result.possibly_assigned_vars.insert(name.clone());
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
            result.tainted_vars.insert(name.clone());
        }

        // Read vars: union — if either branch reads a var, it counts as read
        for name in if_ctx.read_vars.iter().chain(else_ctx.read_vars.iter()) {
            result.read_vars.insert(name.clone());
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
