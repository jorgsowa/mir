//! End-to-end tests for the plugin hook wiring: a compiled-in test plugin is
//! installed into the process-global registry (once — the registry is global,
//! so all tests in this binary share it; keep plugin behavior keyed to
//! distinctive names so tests can't interfere).

use std::sync::Once;

use mir_analyzer::analyze_source;
use mir_issues::{Issue, IssueKind, Severity};
use mir_plugin::php_ast::owned::{ExprKind, StmtKind};
use mir_plugin::{
    AfterExpressionAnalysisEvent, AfterFunctionCallAnalysisEvent, AfterStatementAnalysisEvent,
    ClassPropertyProviderEvent, FunctionReturnTypeProviderEvent, HookFlags,
    MethodReturnTypeProviderEvent, MirPlugin, PluginIssue, PluginRegistry, ProvidedType,
};

struct TestPlugin;

impl MirPlugin for TestPlugin {
    fn name(&self) -> &str {
        "test-plugin"
    }

    fn hooks(&self) -> HookFlags {
        HookFlags {
            after_expression_analysis: true,
            after_statement_analysis: true,
            after_function_call_analysis: true,
            before_add_issue: true,
            ..Default::default()
        }
    }

    fn function_return_type_ids(&self) -> Vec<String> {
        vec!["plugin_helper".to_string()]
    }

    fn function_return_type(
        &self,
        event: &FunctionReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        assert_eq!(event.function_id, "plugin_helper");
        Some(ProvidedType::Parse("non-empty-string".to_string()))
    }

    fn method_return_type_classes(&self) -> Vec<String> {
        vec!["PluginContainer".to_string()]
    }

    fn method_return_type(
        &self,
        event: &MethodReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        (event.method_name == "get").then(|| ProvidedType::Parse("int".to_string()))
    }

    fn class_property_classes(&self) -> Vec<String> {
        vec!["PluginModel".to_string()]
    }

    fn class_property(&self, event: &ClassPropertyProviderEvent<'_>) -> Option<ProvidedType> {
        let casts = event
            .array_property_defaults
            .iter()
            .find(|d| d.property == "casts")?;
        let (_, value) = casts
            .entries
            .iter()
            .find(|(k, _)| k == event.property_name)?;
        Some(ProvidedType::Parse(value.clone()))
    }

    fn after_expression_analysis(&self, event: &mut AfterExpressionAnalysisEvent<'_>) {
        if let ExprKind::Variable(name) = &event.expr.kind {
            if name.as_ref().ends_with("flag_me") {
                event.issues.push(
                    PluginIssue::new("FlaggedVariable", "test plugin flagged this variable")
                        .with_severity(Severity::Warning),
                );
            }
        }
    }

    fn after_statement_analysis(&self, event: &mut AfterStatementAnalysisEvent<'_>) {
        if matches!(event.stmt.kind, StmtKind::Echo(_)) {
            event
                .issues
                .push(PluginIssue::new("EchoSpotted", "echo statement seen"));
            event
                .issues
                .push(PluginIssue::new("VetoedEcho", "should never surface"));
        }
    }

    fn after_function_call_analysis(&self, event: &mut AfterFunctionCallAnalysisEvent<'_>) {
        if event.function_id == "dangerous" {
            event
                .issues
                .push(PluginIssue::new("DangerousCall", "dangerous() is banned"));
        }
    }

    fn before_add_issue(&self, issue: &Issue) -> Option<bool> {
        (issue.kind.display_name() == "VetoedEcho").then_some(false)
    }
}

fn setup() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(TestPlugin));
        mir_plugin::install(registry);
    });
}

fn unsuppressed(source: &str) -> Vec<Issue> {
    analyze_source(source)
        .issues
        .into_iter()
        .filter(|i| !i.suppressed)
        .collect()
}

fn plugin_issue_names(issues: &[Issue]) -> Vec<&str> {
    issues
        .iter()
        .filter(|i| matches!(i.kind, IssueKind::PluginIssue { .. }))
        .map(|i| i.kind.display_name())
        .collect()
}

#[test]
fn after_expression_hook_emits_named_issue() {
    setup();
    let issues = unsuppressed("<?php function f(): void { $flag_me = 1; print $flag_me; }");
    let names = plugin_issue_names(&issues);
    assert!(
        names.contains(&"FlaggedVariable"),
        "expected FlaggedVariable in {names:?}"
    );
    let flagged = issues
        .iter()
        .find(|i| i.kind.display_name() == "FlaggedVariable")
        .unwrap();
    assert_eq!(flagged.severity, Severity::Warning);
    assert_eq!(flagged.kind.name(), "PluginIssue");
}

#[test]
fn function_return_type_provider_overrides_return_type() {
    setup();
    let issues = unsuppressed(
        r#"<?php
function plugin_helper(): string { return "x"; }
function g(): void {
    $x = plugin_helper();
    /** @mir-check $x is non-empty-string */
    print $x;
}
"#,
    );
    let mismatches: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i.kind, IssueKind::TypeCheckMismatch { .. }))
        .collect();
    assert!(
        mismatches.is_empty(),
        "provider should have narrowed string -> non-empty-string: {mismatches:?}"
    );
}

#[test]
fn method_return_type_provider_overrides_return_type() {
    setup();
    let issues = unsuppressed(
        r#"<?php
class PluginContainer {
    public function get(): object { return new stdClass(); }
}
function h(PluginContainer $c): void {
    $x = $c->get();
    /** @mir-check $x is int */
    print $x;
}
"#,
    );
    let mismatches: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i.kind, IssueKind::TypeCheckMismatch { .. }))
        .collect();
    assert!(
        mismatches.is_empty(),
        "method provider should have replaced object -> int: {mismatches:?}"
    );
}

#[test]
fn after_function_call_hook_fires_for_declared_function() {
    setup();
    let issues =
        unsuppressed("<?php function dangerous(): void {} function k(): void { dangerous(); }");
    assert!(
        plugin_issue_names(&issues).contains(&"DangerousCall"),
        "expected DangerousCall in {issues:?}"
    );
}

#[test]
fn plugin_issues_are_suppressible_by_their_own_name() {
    setup();
    let all = analyze_source(
        r#"<?php
function dangerous(): void {}
function k(): void {
    /** @mir-suppress DangerousCall */
    dangerous();
}
"#,
    )
    .issues;
    let dangerous: Vec<_> = all
        .iter()
        .filter(|i| i.kind.display_name() == "DangerousCall")
        .collect();
    assert!(
        dangerous.iter().all(|i| i.suppressed),
        "@mir-suppress DangerousCall should suppress the plugin issue: {dangerous:?}"
    );
}

#[test]
fn before_add_issue_vetoes_and_statement_hook_fires() {
    setup();
    let issues = unsuppressed(r#"<?php function m(): void { echo "hi"; }"#);
    let names = plugin_issue_names(&issues);
    assert!(
        names.contains(&"EchoSpotted"),
        "statement hook should fire: {names:?}"
    );
    assert!(
        !names.contains(&"VetoedEcho"),
        "before_add_issue veto should drop VetoedEcho: {names:?}"
    );
}

#[test]
fn class_property_provider_supplies_undeclared_property_via_ancestor() {
    setup();
    // The marker is registered on the base `PluginModel`, but `$casts` lives on
    // the subclass `PluginUser` — proving ancestor-aware dispatch plus the
    // array-literal-default exposure. `$u->age` must resolve to `int` from the
    // cast entry, with no UndefinedProperty flagged.
    let issues = unsuppressed(
        r#"<?php
class PluginModel {}
class PluginUser extends PluginModel {
    protected $casts = ['age' => 'int'];
}
function h(PluginUser $u): void {
    $x = $u->age;
    /** @mir-check $x is int */
    print $x;
}
"#,
    );
    let undefined: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i.kind, IssueKind::UndefinedProperty { .. }))
        .collect();
    assert!(
        undefined.is_empty(),
        "provider should have supplied $age: {undefined:?}"
    );
    let mismatches: Vec<_> = issues
        .iter()
        .filter(|i| matches!(i.kind, IssueKind::TypeCheckMismatch { .. }))
        .collect();
    assert!(
        mismatches.is_empty(),
        "cast entry should have typed $age as int: {mismatches:?}"
    );
}
