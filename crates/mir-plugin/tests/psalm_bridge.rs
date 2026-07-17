//! End-to-end test of the Psalm plugin bridge against a fixture project that
//! ships a minimal fake of Psalm's plugin API (same interface shapes, no real
//! analysis). Exercises: host spawn, entry-point invocation through the
//! reflection-generated RegistrationInterface shim, stub collection,
//! unsupported-hook warnings, and a live return-type-provider RPC round trip.
//!
//! Skipped (with a note) when no `php` binary is on PATH.
#![cfg(feature = "psalm-bridge")]

use std::path::PathBuf;
use std::process::Command;

use mir_plugin::psalm::{BridgeOptions, PsalmBridgePlugin, PsalmPluginSpec};
use mir_plugin::{
    mir_types, php_ast, FunctionReturnTypeProviderEvent, MirPlugin, ProvidedType, Type,
};

fn php_available() -> bool {
    Command::new("php")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fake-psalm-project")
}

fn provider_event<'a>(
    function_id: &'a str,
    arg_types: &'a [Type],
) -> FunctionReturnTypeProviderEvent<'a> {
    FunctionReturnTypeProviderEvent {
        function_id,
        args: &[],
        arg_types,
        span: php_ast::Span::new(0, 0),
        file: "src/App.php",
        call_snippet: None,
    }
}

#[test]
fn bridge_hosts_a_psalm_plugin_end_to_end() {
    if !php_available() {
        eprintln!("skipping psalm bridge test: no `php` binary on PATH");
        return;
    }

    let bridge = PsalmBridgePlugin::spawn(&BridgeOptions::new(
        fixture_root(),
        vec![PsalmPluginSpec {
            class: "TestPlugin\\Plugin".to_string(),
            config_xml: None,
        }],
    ))
    .expect("bridge should spawn and initialize");

    // Stub registered via RegistrationInterface::addStubFile.
    let stubs = bridge.stub_files();
    assert_eq!(stubs.len(), 1, "stubs: {stubs:?}");
    assert!(
        stubs[0].ends_with("plugin/stubs/helpers.phpstub"),
        "stub path: {:?}",
        stubs[0]
    );

    // FunctionReturnTypeProviderInterface ids collected.
    assert_eq!(bridge.function_return_type_ids(), vec!["test_helper"]);

    // Unsupported hook interface surfaced as a warning, not an error.
    assert!(
        bridge
            .warnings
            .iter()
            .any(|w| w.contains("AfterExpressionAnalysis")),
        "warnings: {:?}",
        bridge.warnings
    );

    // Live provider round trip: the fixture provider wraps the first
    // argument's type (delivered through the NodeTypeProvider shim) in a
    // list<...>.
    let int_arg = [Type::single(mir_types::Atomic::TInt)];
    match bridge.function_return_type(&provider_event("test_helper", &int_arg)) {
        Some(ProvidedType::Parse(s)) => assert_eq!(s, "list<int>"),
        other => panic!("expected Parse type from provider, got {other:?}"),
    }

    // Unknown function id → no override.
    assert!(bridge
        .function_return_type(&provider_event("unrelated_fn", &int_arg))
        .is_none());
}
