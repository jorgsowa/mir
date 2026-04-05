# Spec: Expose TypeEnv and IssueKind for LSP consumers

**Date:** 2026-04-05
**Issue:** #38

## Problem

`php-lsp` currently:
1. Maintains a parallel `TypeMap` (reimplements variable type tracking already done by mir)
2. Filters diagnostics by matching message strings — silently breaks if mir renames a message

Both problems stem from mir not exposing its internal types as a stable public API.

## Design

### New types: `ScopeId` and `TypeEnv`

**File:** `crates/mir-analyzer/src/type_env.rs` (new)

```rust
/// Identifies a single analysis scope within a project.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeId {
    TopLevel { file: Arc<str> },
    Function  { file: Arc<str>, name: Arc<str> },
    Method    { class: Arc<str>, method: Arc<str> },
}

/// Variable type environment for one scope — the stable public view of Context.vars.
pub struct TypeEnv {
    vars: IndexMap<String, Union>,
}

impl TypeEnv {
    pub(crate) fn new(vars: IndexMap<String, Union>) -> Self { Self { vars } }
    pub fn get_var(&self, name: &str) -> Option<&Union> { self.vars.get(name) }
    pub fn var_names(&self) -> impl Iterator<Item = &str> { self.vars.keys().map(|s| s.as_str()) }
}
```

Re-exported from `mir-analyzer/src/lib.rs`:
```rust
pub use type_env::{ScopeId, TypeEnv};
```

### Extended `AnalysisResult`

**File:** `crates/mir-analyzer/src/project.rs`

```rust
pub struct AnalysisResult {
    pub issues: Vec<Issue>,
    pub type_envs: HashMap<ScopeId, TypeEnv>,  // new field
}
```

Collection points (all in `project.rs`):
- **Top-level:** after `analyze_stmts` on the file's top-level body → `ScopeId::TopLevel { file }`
- **`analyze_fn_decl`:** after `sa.analyze_stmts` → `ScopeId::Function { file, name: fqn }`
- **`analyze_method_decl`:** after method body analysis → `ScopeId::Method { class: fqcn, method: name }`

At each collection point:
```rust
result.type_envs.insert(scope_id, TypeEnv::new(ctx.vars.clone()));
```

### IssueKind re-export

**File:** `crates/mir-analyzer/src/lib.rs`

```rust
pub use mir_issues::{Issue, IssueKind, Severity, Location};
```

This gives LSP consumers a single `mir-analyzer` dependency instead of two (`mir-analyzer` + `mir-issues`).

### Convenience method for tests

**File:** `crates/mir-analyzer/src/project.rs`

```rust
impl ProjectAnalyzer {
    /// Analyze a PHP source string without a real file path. Useful for tests and LSP single-file mode.
    pub fn analyze_source(source: &str) -> AnalysisResult {
        let tmp = Arc::from("<source>");
        // parse + Pass 1 + Pass 2 against the single in-memory source
        ...
    }
}
```

## Testing

**File:** `crates/mir-analyzer/tests/type_env.rs`

| Test | Scenario |
|------|----------|
| `returns_type_env_for_top_level_scope` | `$x = 1;` at top level → `TypeEnv` contains `x: int` |
| `returns_type_env_for_function_scope` | `function f() { $y = "hi"; }` → `ScopeId::Function` env contains `y: string` |
| `returns_type_env_for_method_scope` | Class method with `$z = true;` → `ScopeId::Method` env contains `z: bool` |
| `get_var_returns_none_for_unknown_variable` | `get_var("nonexistent")` → `None` |
| `var_names_lists_all_variables_in_scope` | Two vars in scope → `var_names()` yields both |

## Acceptance criteria

- `result.type_envs` is populated for every analyzed function, method, and top-level body
- `IssueKind` accessible via `mir_analyzer::IssueKind` (no direct `mir_issues` dep needed)
- All existing tests pass unchanged
- All 5 new `type_env` tests pass
