// crates/mir-analyzer/src/type_env.rs
use mir_types::{Name, Type};
use std::sync::Arc;

/// Identifies a single analysis scope within a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeId {
    Function { file: Arc<str>, name: Arc<str> },
    Method { class: Arc<str>, method: Arc<str> },
}

/// Variable type environment for one scope — the stable public view of FlowState.vars.
#[allow(dead_code)]
#[derive(Debug)]
pub struct TypeEnv {
    #[allow(dead_code)]
    vars: Arc<rustc_hash::FxHashMap<Name, Arc<Type>>>,
}

impl TypeEnv {
    pub(crate) fn new(vars: Arc<rustc_hash::FxHashMap<Name, Arc<Type>>>) -> Self {
        Self { vars }
    }

    /// Returns the inferred type of `$name`, or `None` if the variable was not tracked.
    #[allow(dead_code)]
    pub fn get_var(&self, name: &str) -> Option<&Type> {
        let sym = Name::from(name);
        self.vars.get(&sym).map(|arc| arc.as_ref())
    }

    /// Iterates over all variable names tracked in this scope.
    #[allow(dead_code)]
    pub fn var_names(&self) -> impl Iterator<Item = &str> {
        self.vars.keys().map(|s| s.as_str())
    }
}
