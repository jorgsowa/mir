//! Bridge for reusing existing Psalm PHP plugins.
//!
//! Psalm plugins are PHP classes implementing
//! `Psalm\Plugin\PluginEntryPointInterface`, installed via composer alongside
//! `vimeo/psalm`. This bridge spawns a long-lived `php` subprocess (the host
//! script embedded in this crate) that boots the analyzed project's
//! `vendor/autoload.php`, invokes each configured entry point against a shim
//! `RegistrationInterface`, and answers JSON-lines RPC from mir.
//!
//! ## Supported Psalm plugin capabilities (v1)
//! - `RegistrationInterface::addStubFile` — full support; stubs feed mir's
//!   normal stub loading.
//! - `FunctionReturnTypeProviderInterface` / `MethodReturnTypeProviderInterface`
//!   — best effort: the host reconstructs the event from the call snippet and
//!   argument types mir sends; provider results are cached per call signature.
//!
//! Other hook registrations (`AfterExpressionAnalysis`, taint hooks, …) are
//! reported in [`PsalmBridgePlugin::warnings`] and skipped — they would need
//! per-node RPC and a full Psalm `Codebase` shim.

use std::io::{BufRead, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use crate::{
    FunctionReturnTypeProviderEvent, MethodReturnTypeProviderEvent, MirPlugin, ProvidedType,
};

/// The PHP host program, embedded so the mir binary is self-contained. It is
/// materialized next to the cache (or in the OS temp dir) at spawn time.
const HOST_PHP: &str = include_str!("host.php");

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("cannot write psalm plugin host script: {0}")]
    WriteHost(std::io::Error),
    #[error("cannot spawn `{php}`: {source} (is PHP installed and on PATH?)")]
    Spawn { php: String, source: std::io::Error },
    #[error("psalm plugin host: {0}")]
    Host(String),
    #[error("psalm plugin host i/o: {0}")]
    Io(#[from] std::io::Error),
    #[error("psalm plugin host protocol: {0}")]
    Protocol(String),
}

/// One `<pluginClass class="..."/>` entry from mir.xml / psalm.xml.
#[derive(Debug, Clone)]
pub struct PsalmPluginSpec {
    /// Fully-qualified entry-point class, e.g. `Psalm\PhpUnitPlugin\Plugin`.
    pub class: String,
    /// Inner XML of the `<pluginClass>` element, passed to the entry point as
    /// its `SimpleXMLElement` config (Psalm's `pluginSpecificConfig`).
    pub config_xml: Option<String>,
}

/// Options for spawning the bridge.
#[derive(Debug, Clone)]
pub struct BridgeOptions {
    /// PHP CLI binary. Defaults to `"php"` on PATH.
    pub php_binary: String,
    /// Project root containing `vendor/autoload.php`.
    pub project_root: PathBuf,
    /// Directory to materialize the host script into. Falls back to the OS
    /// temp dir when `None`.
    pub host_script_dir: Option<PathBuf>,
    pub plugins: Vec<PsalmPluginSpec>,
}

impl BridgeOptions {
    pub fn new(project_root: impl Into<PathBuf>, plugins: Vec<PsalmPluginSpec>) -> Self {
        Self {
            php_binary: "php".to_string(),
            project_root: project_root.into(),
            host_script_dir: None,
            plugins,
        }
    }
}

struct Rpc {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Rpc {
    fn call(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, BridgeError> {
        self.next_id += 1;
        let id = self.next_id;
        let request = serde_json::json!({ "id": id, "method": method, "params": params });
        serde_json::to_writer(&mut self.stdin, &request)
            .map_err(|e| BridgeError::Protocol(e.to_string()))?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;

        let mut line = String::new();
        loop {
            line.clear();
            if self.stdout.read_line(&mut line)? == 0 {
                return Err(BridgeError::Protocol("host exited unexpectedly".into()));
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            // Skip stray output that is not our response (echoing plugins).
            let Ok(response) = serde_json::from_str::<serde_json::Value>(trimmed) else {
                continue;
            };
            if response.get("id").and_then(|v| v.as_u64()) != Some(id) {
                continue;
            }
            if let Some(err) = response.get("error").and_then(|v| v.as_str()) {
                return Err(BridgeError::Host(err.to_string()));
            }
            return Ok(response.get("result").cloned().unwrap_or_default());
        }
    }
}

impl Drop for Rpc {
    fn drop(&mut self) {
        let _ = self
            .stdin
            .write_all(b"{\"id\":0,\"method\":\"shutdown\",\"params\":{}}\n");
        let _ = self.stdin.flush();
        let _ = self.child.wait();
    }
}

/// A [`MirPlugin`] that proxies to Psalm PHP plugins running in the host
/// subprocess. Register it into the [`crate::PluginRegistry`] like any other
/// plugin.
pub struct PsalmBridgePlugin {
    rpc: Mutex<Rpc>,
    /// Set after an unrecoverable RPC failure; all further queries return
    /// `None` so analysis degrades to normal inference instead of erroring
    /// on every call site.
    dead: AtomicBool,
    stubs: Vec<PathBuf>,
    function_ids: Vec<String>,
    method_classes: Vec<String>,
    /// Unsupported-hook and host-side setup warnings, for the CLI to print.
    pub warnings: Vec<String>,
    /// provider-result cache: call-signature key → docblock type string.
    cache: Mutex<FxHashMap<String, Option<String>>>,
}

impl PsalmBridgePlugin {
    /// Spawn the PHP host, run the plugins' entry points, and collect what
    /// they registered.
    pub fn spawn(options: &BridgeOptions) -> Result<Self, BridgeError> {
        let script = materialize_host_script(options.host_script_dir.as_deref())?;

        let mut child = Command::new(&options.php_binary)
            .arg("-d")
            .arg("display_errors=stderr")
            .arg(&script)
            .current_dir(&options.project_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|source| BridgeError::Spawn {
                php: options.php_binary.clone(),
                source,
            })?;

        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let mut rpc = Rpc {
            child,
            stdin,
            stdout,
            next_id: 0,
        };

        let plugins: Vec<serde_json::Value> = options
            .plugins
            .iter()
            .map(|p| serde_json::json!({ "class": p.class, "configXml": p.config_xml }))
            .collect();
        let init = rpc.call(
            "init",
            serde_json::json!({
                "projectRoot": options.project_root.to_string_lossy(),
                "plugins": plugins,
            }),
        )?;

        let str_list = |key: &str| -> Vec<String> {
            init.get(key)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };

        Ok(Self {
            rpc: Mutex::new(rpc),
            dead: AtomicBool::new(false),
            stubs: str_list("stubs").into_iter().map(PathBuf::from).collect(),
            function_ids: str_list("functionIds"),
            method_classes: str_list("methodClasses"),
            warnings: str_list("warnings"),
            cache: Mutex::new(FxHashMap::default()),
        })
    }

    /// Whether the plugins registered anything mir can actually use.
    pub fn is_effectively_empty(&self) -> bool {
        self.stubs.is_empty() && self.function_ids.is_empty() && self.method_classes.is_empty()
    }

    fn query_type(
        &self,
        method: &str,
        cache_key: String,
        params: serde_json::Value,
    ) -> Option<ProvidedType> {
        if self.dead.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(cached) = self.cache.lock().get(&cache_key) {
            return cached.clone().map(ProvidedType::Parse);
        }
        let result = self.rpc.lock().call(method, params);
        let type_string = match result {
            Ok(value) => value
                .get("type")
                .and_then(|v| v.as_str())
                .map(str::to_string),
            Err(e) => {
                if !self.dead.swap(true, Ordering::Relaxed) {
                    eprintln!("mir: psalm plugin bridge disabled after error: {e}");
                }
                return None;
            }
        };
        self.cache.lock().insert(cache_key, type_string.clone());
        type_string.map(ProvidedType::Parse)
    }
}

fn type_strings(types: &[crate::Type]) -> Vec<String> {
    types.iter().map(|t| t.to_string()).collect()
}

fn signature_key(head: &str, arg_types: &[crate::Type]) -> String {
    let mut key = String::from(head);
    for t in arg_types {
        key.push('\u{1f}');
        key.push_str(&t.to_string());
    }
    key
}

impl MirPlugin for PsalmBridgePlugin {
    fn name(&self) -> &str {
        "psalm-bridge"
    }

    fn stub_files(&self) -> Vec<PathBuf> {
        self.stubs.clone()
    }

    fn function_return_type_ids(&self) -> Vec<String> {
        self.function_ids.clone()
    }

    fn function_return_type(
        &self,
        event: &FunctionReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        self.query_type(
            "functionReturnType",
            signature_key(event.function_id, event.arg_types),
            serde_json::json!({
                "functionId": event.function_id,
                "argTypes": type_strings(event.arg_types),
                "snippet": event.call_snippet,
                "file": event.file,
            }),
        )
    }

    fn method_return_type_classes(&self) -> Vec<String> {
        self.method_classes.clone()
    }

    fn method_return_type(
        &self,
        event: &MethodReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        self.query_type(
            "methodReturnType",
            signature_key(
                &format!("{}::{}", event.fqcn, event.method_name),
                event.arg_types,
            ),
            serde_json::json!({
                "fqcn": event.fqcn,
                "methodName": event.method_name,
                "argTypes": type_strings(event.arg_types),
                "snippet": event.call_snippet,
                "file": event.file,
            }),
        )
    }
}

/// Write the embedded host script to disk (content-addressed name so
/// concurrent mir processes and version upgrades never clash) and return its
/// path.
fn materialize_host_script(dir: Option<&Path>) -> Result<PathBuf, BridgeError> {
    let dir = dir
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&dir).map_err(BridgeError::WriteHost)?;
    let digest = content_hash_hex(HOST_PHP);
    let path = dir.join(format!("mir-psalm-host-{digest}.php"));
    if !path.exists() {
        let tmp = dir.join(format!("mir-psalm-host-{digest}.php.tmp.{}", std::process::id()));
        std::fs::write(&tmp, HOST_PHP).map_err(BridgeError::WriteHost)?;
        std::fs::rename(&tmp, &path).map_err(BridgeError::WriteHost)?;
    }
    Ok(path)
}

/// Tiny stable content hash (FNV-1a over the script) — collision resistance
/// beyond "different versions get different names" is not needed here.
fn content_hash_hex(content: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in content.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}
