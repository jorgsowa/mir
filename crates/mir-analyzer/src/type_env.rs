// crates/mir-analyzer/src/type_env.rs
use std::sync::Arc;
use indexmap::IndexMap;
use mir_types::Union;

/// Identifies a single analysis scope within a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeId {
    Function { file: Arc<str>, name: Arc<str> },
    Method   { class: Arc<str>, method: Arc<str> },
}

/// Variable type environment for one scope — the stable public view of Context.vars.
pub struct TypeEnv {
    vars: IndexMap<String, Union>,
}

impl TypeEnv {
    pub(crate) fn new(vars: IndexMap<String, Union>) -> Self {
        Self { vars }
    }

    /// Returns the inferred type of `$name`, or `None` if the variable was not tracked.
    pub fn get_var(&self, name: &str) -> Option<&Union> {
        self.vars.get(name)
    }

    /// Iterates over all variable names tracked in this scope.
    pub fn var_names(&self) -> impl Iterator<Item = &str> {
        self.vars.keys().map(|s| s.as_str())
    }
}
