/// Plugin bootstrap: build and install the global plugin registry from
/// `<plugins>` config before any analysis session is created.
use std::path::{Path, PathBuf};
use std::process;

use mir_plugin::psalm::{BridgeOptions, PsalmBridgePlugin, PsalmPluginSpec};
use mir_plugin::PluginRegistry;

use crate::config::Config;
use crate::Cli;

/// Load Rust dylib plugins and spawn the Psalm bridge (when configured),
/// install the registry, and fold plugin-contributed stub files back into
/// `config.stub_files` so the analysis session loads them like user stubs.
///
/// A plugin that is explicitly configured but fails to load is a hard error
/// (exit 2), matching how Psalm treats broken plugins — silently analyzing
/// without a requested plugin would produce misleadingly green results.
pub fn setup_plugins(
    cli: &Cli,
    config: &mut Config,
    config_base: &Path,
    composer_root: Option<&Path>,
) {
    if config.psalm_plugins.is_empty() && config.rust_plugins.is_empty() {
        return;
    }

    let mut registry = PluginRegistry::new();

    for raw in &config.rust_plugins {
        let path = resolve(raw, config_base);
        match mir_plugin::dylib::load(&path) {
            Ok(plugin) => {
                if !cli.quiet {
                    eprintln!("mir: loaded rust plugin '{}'", plugin.name());
                }
                registry.register(plugin);
            }
            Err(e) => {
                eprintln!("mir: {e}");
                process::exit(2);
            }
        }
    }

    if !config.psalm_plugins.is_empty() {
        let specs: Vec<PsalmPluginSpec> = config
            .psalm_plugins
            .iter()
            .map(|p| PsalmPluginSpec {
                class: p.class.clone(),
                config_xml: p.config_xml.clone(),
            })
            .collect();
        // vendor/autoload.php lives at the composer root; fall back to the
        // config file's directory for non-composer layouts.
        let project_root = composer_root.unwrap_or(config_base);
        let mut options = BridgeOptions::new(project_root, specs);
        if let Ok(php) = std::env::var("MIR_PHP") {
            options.php_binary = php;
        }
        match PsalmBridgePlugin::spawn(&options) {
            Ok(bridge) => {
                if !cli.quiet {
                    for warning in &bridge.warnings {
                        eprintln!("mir: psalm plugin: {warning}");
                    }
                    eprintln!(
                        "mir: psalm plugins active ({} classes)",
                        config.psalm_plugins.len()
                    );
                }
                if !cli.quiet && bridge.is_effectively_empty() {
                    eprintln!(
                        "mir: psalm plugins registered nothing mir can use (see warnings above)"
                    );
                }
                registry.register(Box::new(bridge));
            }
            Err(e) => {
                eprintln!("mir: psalm plugin bridge: {e}");
                process::exit(2);
            }
        }
    }

    // Stubs from every plugin (Rust and Psalm alike) feed the session's
    // user-stub loading, so they take precedence over vendor definitions.
    for stub in registry.stub_files() {
        config.stub_files.push(stub.to_string_lossy().into_owned());
    }

    mir_plugin::install(registry);
}

fn resolve(raw: &str, base: &Path) -> PathBuf {
    let p = PathBuf::from(raw);
    if p.is_absolute() {
        p
    } else {
        base.join(p)
    }
}
