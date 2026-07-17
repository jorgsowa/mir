//! Example mir plugin: bans `var_dump()`/`dd()` calls and teaches mir the
//! precise return type of a hypothetical `app_config()` helper.
//!
//! Build (`cargo build -p mir-plugin-example --release`), then enable in
//! `mir.xml`:
//!
//! ```xml
//! <plugins>
//!     <rustPlugin path="target/release/libmir_plugin_example.dylib"/>
//! </plugins>
//! ```
//!
//! The dylib must be built with the same Rust toolchain and mir-plugin
//! version as the mir binary loading it.

use mir_plugin::{
    AfterFunctionCallAnalysisEvent, FunctionReturnTypeProviderEvent, HookFlags, MirPlugin,
    PluginIssue, ProvidedType, Severity,
};

struct ExamplePlugin;

impl MirPlugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example"
    }

    fn hooks(&self) -> HookFlags {
        HookFlags {
            after_function_call_analysis: true,
            ..Default::default()
        }
    }

    fn after_function_call_analysis(&self, event: &mut AfterFunctionCallAnalysisEvent<'_>) {
        if matches!(event.function_id, "var_dump" | "dd" | "dump") {
            event.issues.push(
                PluginIssue::new(
                    "DebugFunctionCall",
                    format!("{}() must not be committed", event.function_id),
                )
                .with_severity(Severity::Warning),
            );
        }
    }

    fn function_return_type_ids(&self) -> Vec<String> {
        vec!["app_config".to_string()]
    }

    fn function_return_type(
        &self,
        event: &FunctionReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        // A literal-string first argument selects the config key; pretend
        // string keys are always strings and everything else is mixed.
        let key_is_string = event.arg_types.first().is_some_and(|t| {
            t.types
                .iter()
                .any(|a| matches!(a, mir_plugin::mir_types::Atomic::TLiteralString(_)))
        });
        key_is_string.then(|| ProvidedType::Parse("non-empty-string".to_string()))
    }
}

fn create() -> Box<dyn MirPlugin> {
    Box::new(ExamplePlugin)
}

mir_plugin::export_plugin!(create);
