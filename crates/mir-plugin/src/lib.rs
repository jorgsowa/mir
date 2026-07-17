//! Plugin API for the mir PHP static analyzer, modeled on Psalm's plugin
//! event handlers (<https://psalm.dev/docs/running_psalm/plugins/plugins_overview/>).
//!
//! A plugin implements [`MirPlugin`], declares which hooks it wants via
//! [`MirPlugin::hooks`], and is registered into a [`PluginRegistry`] that the
//! host process installs globally with [`install`]. The analyzer snapshots the
//! registry once per analysis pass; when no registry is installed every hook
//! site reduces to a single `Option` check.
//!
//! Plugins come in two flavors:
//! - **Rust plugins** — compiled in (registered directly) or loaded from a
//!   cdylib at runtime (`dylib` feature, see [`dylib`]).
//! - **Psalm PHP plugins** — reused through a PHP host subprocess
//!   (`psalm-bridge` feature, see [`psalm`]).

use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

pub use mir_issues::{Issue, Severity};
pub use mir_types::Type;
// Re-exported so plugin crates match AST nodes and build types without
// pinning the underlying crates themselves.
pub use mir_types;
pub use php_ast;

#[cfg(feature = "dylib")]
pub mod dylib;
#[cfg(feature = "psalm-bridge")]
pub mod psalm;

/// Bumped whenever the [`MirPlugin`] trait or event types change incompatibly.
/// Dylib plugins built against a different version are refused at load time.
pub const MIR_PLUGIN_API_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Issues emitted by plugins
// ---------------------------------------------------------------------------

/// An issue raised by a plugin. Converted by the analyzer into
/// `IssueKind::PluginIssue` with a proper source `Location`.
#[derive(Debug, Clone)]
pub struct PluginIssue {
    /// Issue name used for display and suppression matching
    /// (`@mir-suppress MyIssueName`, `<MyIssueName errorLevel="suppress"/>`).
    pub name: String,
    pub message: String,
    pub severity: Severity,
    /// Span the issue points at. `None` means the span of the event's node.
    pub span: Option<php_ast::Span>,
}

impl PluginIssue {
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
            severity: Severity::Error,
            span: None,
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_span(mut self, span: php_ast::Span) -> Self {
        self.span = Some(span);
        self
    }
}

// ---------------------------------------------------------------------------
// Provided types
// ---------------------------------------------------------------------------

/// A type contributed by a plugin. `Parse` carries a docblock-syntax type
/// string (e.g. `list<non-empty-string>`) that the analyzer resolves with its
/// own type parser in the context of the analyzed file — this is what the
/// Psalm bridge returns, since PHP-side plugins produce type strings.
#[derive(Debug, Clone)]
pub enum ProvidedType {
    Union(Type),
    Parse(String),
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Counterpart of Psalm's `AfterExpressionAnalysisEvent`.
pub struct AfterExpressionAnalysisEvent<'a> {
    pub expr: &'a php_ast::owned::Expr,
    /// Type the analyzer inferred for the expression.
    pub expr_type: &'a Type,
    pub file: &'a str,
    pub issues: Vec<PluginIssue>,
}

/// Counterpart of Psalm's `AfterStatementAnalysisEvent`.
pub struct AfterStatementAnalysisEvent<'a> {
    pub stmt: &'a php_ast::owned::Stmt,
    pub file: &'a str,
    pub issues: Vec<PluginIssue>,
}

/// Counterpart of Psalm's `AfterFunctionCallAnalysisEvent`. `return_type`
/// starts as the analyzer's inferred type and may be replaced.
pub struct AfterFunctionCallAnalysisEvent<'a> {
    /// Lowercased fully-qualified function name without leading `\`.
    pub function_id: &'a str,
    pub args: &'a [php_ast::owned::Arg],
    pub arg_types: &'a [Type],
    pub span: php_ast::Span,
    pub file: &'a str,
    pub return_type: &'a mut Type,
    pub issues: Vec<PluginIssue>,
}

/// Counterpart of Psalm's `AfterMethodCallAnalysisEvent`.
pub struct AfterMethodCallAnalysisEvent<'a> {
    /// `Fully\Qualified\Class::methodname` (class as declared, method lowercased).
    pub method_id: &'a str,
    pub args: &'a [php_ast::owned::Arg],
    pub arg_types: &'a [Type],
    pub span: php_ast::Span,
    pub file: &'a str,
    pub return_type: &'a mut Type,
    pub issues: Vec<PluginIssue>,
}

/// Counterpart of Psalm's `FunctionReturnTypeProviderEvent`.
pub struct FunctionReturnTypeProviderEvent<'a> {
    /// Lowercased fully-qualified function name without leading `\`.
    pub function_id: &'a str,
    pub args: &'a [php_ast::owned::Arg],
    pub arg_types: &'a [Type],
    pub span: php_ast::Span,
    pub file: &'a str,
    /// Raw source text of the whole call expression, when available. The
    /// Psalm bridge re-parses this on the PHP side to build genuine
    /// `PhpParser` argument nodes for the wrapped plugin.
    pub call_snippet: Option<&'a str>,
}

/// Counterpart of Psalm's `MethodReturnTypeProviderEvent`.
pub struct MethodReturnTypeProviderEvent<'a> {
    /// FQCN of the class the method was resolved on (no leading `\`).
    pub fqcn: &'a str,
    /// Lowercased method name (PHP method dispatch is case-insensitive).
    pub method_name: &'a str,
    pub args: &'a [php_ast::owned::Arg],
    pub arg_types: &'a [Type],
    pub span: php_ast::Span,
    pub file: &'a str,
    pub call_snippet: Option<&'a str>,
}

/// Counterpart of Psalm's `AfterCodebasePopulatedEvent`. Fired once per batch
/// run after definition collection, before body analysis.
pub struct AfterCodebasePopulatedEvent<'a> {
    /// Files that were indexed in this pass.
    pub files: &'a [Arc<str>],
}

// ---------------------------------------------------------------------------
// Plugin trait
// ---------------------------------------------------------------------------

/// Which event hooks a plugin subscribes to. Sites only construct events and
/// dispatch when at least one registered plugin set the matching flag, so an
/// unset flag keeps that hook zero-cost.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HookFlags {
    pub after_expression_analysis: bool,
    pub after_statement_analysis: bool,
    pub after_function_call_analysis: bool,
    pub after_method_call_analysis: bool,
    pub before_add_issue: bool,
    pub after_codebase_populated: bool,
}

/// A mir plugin. Hook methods take `&self` and may run concurrently from
/// rayon workers — use interior mutability with proper synchronization.
///
/// Return-type providers are separate from [`HookFlags`]: they are keyed by
/// the ids returned from [`function_return_type_ids`] /
/// [`method_return_type_classes`], mirroring Psalm's
/// `FunctionReturnTypeProviderInterface::getFunctionIds()` and
/// `MethodReturnTypeProviderInterface::getClassLikeNames()`.
///
/// [`function_return_type_ids`]: MirPlugin::function_return_type_ids
/// [`method_return_type_classes`]: MirPlugin::method_return_type_classes
pub trait MirPlugin: Send + Sync {
    fn name(&self) -> &str;

    fn hooks(&self) -> HookFlags {
        HookFlags::default()
    }

    /// PHP stub files this plugin contributes (Psalm's
    /// `RegistrationInterface::addStubFile`). Loaded before analysis.
    fn stub_files(&self) -> Vec<PathBuf> {
        Vec::new()
    }

    /// Function ids (lowercased FQNs, no leading `\`) this plugin provides
    /// return types for.
    fn function_return_type_ids(&self) -> Vec<String> {
        Vec::new()
    }

    /// Override the return type of a call to one of the declared function
    /// ids. `None` falls through to the next plugin / normal inference.
    fn function_return_type(
        &self,
        _event: &FunctionReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        None
    }

    /// Class FQCNs (no leading `\`) this plugin provides method return types
    /// for.
    fn method_return_type_classes(&self) -> Vec<String> {
        Vec::new()
    }

    fn method_return_type(
        &self,
        _event: &MethodReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        None
    }

    fn after_expression_analysis(&self, _event: &mut AfterExpressionAnalysisEvent<'_>) {}

    fn after_statement_analysis(&self, _event: &mut AfterStatementAnalysisEvent<'_>) {}

    fn after_function_call_analysis(&self, _event: &mut AfterFunctionCallAnalysisEvent<'_>) {}

    fn after_method_call_analysis(&self, _event: &mut AfterMethodCallAnalysisEvent<'_>) {}

    /// Veto or pass an issue before it is reported (Psalm's
    /// `BeforeAddIssueInterface`). `Some(false)` drops the issue, `Some(true)`
    /// forces it through, `None` defers to other plugins.
    fn before_add_issue(&self, _issue: &Issue) -> Option<bool> {
        None
    }

    fn after_codebase_populated(&self, _event: &mut AfterCodebasePopulatedEvent<'_>) {}
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Normalize a function id or FQCN for provider-map lookup: lowercase, no
/// leading backslash.
pub fn normalize_id(id: &str) -> String {
    id.trim_start_matches('\\').to_ascii_lowercase()
}

#[derive(Default)]
pub struct PluginRegistry {
    plugins: Vec<Box<dyn MirPlugin>>,
    combined_hooks: HookFlags,
    /// normalized function id → plugin indices, in registration order.
    function_providers: FxHashMap<String, Vec<usize>>,
    /// normalized FQCN → plugin indices, in registration order.
    method_providers: FxHashMap<String, Vec<usize>>,
    /// Indices of plugins subscribed to each hook, so dispatch skips
    /// non-subscribers without a virtual call.
    after_expr: Vec<usize>,
    after_stmt: Vec<usize>,
    after_fn_call: Vec<usize>,
    after_method_call: Vec<usize>,
    before_issue: Vec<usize>,
    after_codebase: Vec<usize>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: Box<dyn MirPlugin>) {
        let idx = self.plugins.len();
        let hooks = plugin.hooks();
        macro_rules! subscribe {
            ($flag:ident, $list:ident) => {
                if hooks.$flag {
                    self.combined_hooks.$flag = true;
                    self.$list.push(idx);
                }
            };
        }
        subscribe!(after_expression_analysis, after_expr);
        subscribe!(after_statement_analysis, after_stmt);
        subscribe!(after_function_call_analysis, after_fn_call);
        subscribe!(after_method_call_analysis, after_method_call);
        subscribe!(before_add_issue, before_issue);
        subscribe!(after_codebase_populated, after_codebase);

        for id in plugin.function_return_type_ids() {
            self.function_providers
                .entry(normalize_id(&id))
                .or_default()
                .push(idx);
        }
        for fqcn in plugin.method_return_type_classes() {
            self.method_providers
                .entry(normalize_id(&fqcn))
                .or_default()
                .push(idx);
        }
        self.plugins.push(plugin);
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn plugin_names(&self) -> Vec<&str> {
        self.plugins.iter().map(|p| p.name()).collect()
    }

    pub fn hooks(&self) -> HookFlags {
        self.combined_hooks
    }

    /// All stub files contributed by registered plugins.
    pub fn stub_files(&self) -> Vec<PathBuf> {
        self.plugins.iter().flat_map(|p| p.stub_files()).collect()
    }

    /// Whether any plugin provides a return type for `function_id`
    /// (pre-normalized). Cheap gate before building the provider event.
    pub fn has_function_provider(&self, function_id: &str) -> bool {
        self.function_providers.contains_key(function_id)
    }

    pub fn has_method_provider(&self, fqcn_normalized: &str) -> bool {
        self.method_providers.contains_key(fqcn_normalized)
    }

    /// Whether any registered plugin declares any return-type provider —
    /// used to skip id normalization entirely on the hot call path.
    pub fn has_any_function_provider(&self) -> bool {
        !self.function_providers.is_empty()
    }

    pub fn has_any_method_provider(&self) -> bool {
        !self.method_providers.is_empty()
    }

    /// First-plugin-wins return type for a function call, in registration
    /// order (matching Psalm, where the last registered provider for an id
    /// replaces earlier ones — we instead chain until one returns `Some`).
    pub fn function_return_type(
        &self,
        event: &FunctionReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        let indices = self.function_providers.get(event.function_id)?;
        indices
            .iter()
            .find_map(|&i| self.plugins[i].function_return_type(event))
    }

    pub fn method_return_type(
        &self,
        fqcn_normalized: &str,
        event: &MethodReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        let indices = self.method_providers.get(fqcn_normalized)?;
        indices
            .iter()
            .find_map(|&i| self.plugins[i].method_return_type(event))
    }

    pub fn after_expression_analysis(&self, event: &mut AfterExpressionAnalysisEvent<'_>) {
        for &i in &self.after_expr {
            self.plugins[i].after_expression_analysis(event);
        }
    }

    pub fn after_statement_analysis(&self, event: &mut AfterStatementAnalysisEvent<'_>) {
        for &i in &self.after_stmt {
            self.plugins[i].after_statement_analysis(event);
        }
    }

    pub fn after_function_call_analysis(&self, event: &mut AfterFunctionCallAnalysisEvent<'_>) {
        for &i in &self.after_fn_call {
            self.plugins[i].after_function_call_analysis(event);
        }
    }

    pub fn after_method_call_analysis(&self, event: &mut AfterMethodCallAnalysisEvent<'_>) {
        for &i in &self.after_method_call {
            self.plugins[i].after_method_call_analysis(event);
        }
    }

    /// `false` when some plugin vetoed the issue. First non-`None` wins.
    pub fn before_add_issue(&self, issue: &Issue) -> bool {
        for &i in &self.before_issue {
            if let Some(keep) = self.plugins[i].before_add_issue(issue) {
                return keep;
            }
        }
        true
    }

    pub fn after_codebase_populated(&self, event: &mut AfterCodebasePopulatedEvent<'_>) {
        for &i in &self.after_codebase {
            self.plugins[i].after_codebase_populated(event);
        }
    }
}

// ---------------------------------------------------------------------------
// Process-global registry
// ---------------------------------------------------------------------------

static REGISTRY: RwLock<Option<Arc<PluginRegistry>>> = RwLock::new(None);

/// Install the process-wide plugin registry. The analyzer takes an `Arc`
/// snapshot per pass, so re-installing affects subsequent passes only.
pub fn install(registry: PluginRegistry) {
    let shared = if registry.is_empty() {
        None
    } else {
        Some(Arc::new(registry))
    };
    *REGISTRY.write() = shared;
}

/// Snapshot the installed registry. `None` when no plugins are loaded — the
/// common case, which every hook site checks first.
pub fn snapshot() -> Option<Arc<PluginRegistry>> {
    REGISTRY.read().clone()
}

/// Remove the installed registry (used by tests).
#[doc(hidden)]
pub fn uninstall() {
    *REGISTRY.write() = None;
}

// ---------------------------------------------------------------------------
// Dylib plugin declaration (the exported entry point lives here so the macro
// works without the `dylib` feature — only *loading* needs libloading).
// ---------------------------------------------------------------------------

/// Entry-point record a Rust cdylib plugin exports under the symbol
/// `MIR_PLUGIN_DECLARATION`. Use [`export_plugin!`] instead of writing this
/// by hand.
#[repr(C)]
pub struct PluginDeclaration {
    pub api_version: u32,
    pub create: fn() -> Box<dyn MirPlugin>,
}

/// Export a plugin constructor from a cdylib crate:
///
/// ```ignore
/// fn create() -> Box<dyn mir_plugin::MirPlugin> { Box::new(MyPlugin) }
/// mir_plugin::export_plugin!(create);
/// ```
///
/// The dylib must be built with the same Rust toolchain and mir-plugin
/// version as the mir binary that loads it — the loader refuses mismatched
/// `api_version`s, but layout compatibility beyond that is on the builder.
#[macro_export]
macro_rules! export_plugin {
    ($create:path) => {
        #[no_mangle]
        pub static MIR_PLUGIN_DECLARATION: $crate::PluginDeclaration = $crate::PluginDeclaration {
            api_version: $crate::MIR_PLUGIN_API_VERSION,
            create: $create,
        };
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoopPlugin;
    impl MirPlugin for NoopPlugin {
        fn name(&self) -> &str {
            "noop"
        }
    }

    struct ExprPlugin;
    impl MirPlugin for ExprPlugin {
        fn name(&self) -> &str {
            "expr"
        }
        fn hooks(&self) -> HookFlags {
            HookFlags {
                after_expression_analysis: true,
                ..Default::default()
            }
        }
        fn function_return_type_ids(&self) -> Vec<String> {
            vec!["\\App\\helper".to_string()]
        }
        fn function_return_type(
            &self,
            _event: &FunctionReturnTypeProviderEvent<'_>,
        ) -> Option<ProvidedType> {
            Some(ProvidedType::Parse("non-empty-string".to_string()))
        }
    }

    #[test]
    fn registry_indexes_hooks_and_providers() {
        let mut reg = PluginRegistry::new();
        reg.register(Box::new(NoopPlugin));
        reg.register(Box::new(ExprPlugin));

        assert_eq!(reg.len(), 2);
        assert!(reg.hooks().after_expression_analysis);
        assert!(!reg.hooks().after_statement_analysis);
        assert!(reg.has_function_provider("app\\helper"));
        assert!(!reg.has_function_provider("app\\other"));
        assert!(reg.has_any_function_provider());
        assert!(!reg.has_any_method_provider());
    }

    #[test]
    fn normalize_id_strips_backslash_and_lowercases() {
        assert_eq!(normalize_id("\\App\\Helper"), "app\\helper");
        assert_eq!(normalize_id("strlen"), "strlen");
    }
}
