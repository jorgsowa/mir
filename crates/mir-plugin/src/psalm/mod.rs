//! Bridge for reusing existing Psalm PHP plugins.
//!
//! Psalm plugins are PHP classes implementing
//! `Psalm\Plugin\PluginEntryPointInterface`, installed via composer alongside
//! `vimeo/psalm`. This bridge spawns a long-lived `php` subprocess (the host
//! script embedded in this crate) that boots the analyzed project's
//! `vendor/autoload.php` (plus a separately installed Psalm, when the project
//! autoloader lacks it), invokes each configured entry point against a shim
//! `RegistrationInterface`, and answers JSON-lines RPC from mir.
//!
//! ## Supported Psalm plugin capabilities
//! - `RegistrationInterface::addStubFile` — full support; stubs feed mir's
//!   normal stub loading.
//! - File-based plugins (`<plugin filename>`): the file is required and its
//!   first declared class is registered as a hook class, like Psalm does.
//! - `FunctionReturnTypeProviderInterface` / `MethodReturnTypeProviderInterface`
//!   — best effort: the host reconstructs the event from the call snippet and
//!   argument types mir sends; provider results are cached per call signature
//!   and calling class.
//! - `AfterFunctionLikeAnalysisInterface` — fired per named function and
//!   method with real Psalm storage objects built from mir's declarations
//!   (params, declared types, parameter attributes).
//! - `AfterClassLikeVisitInterface`, `AfterClassLikeAnalysisInterface` and
//!   `AfterCodebasePopulatedInterface` — fired during batch runs, once per
//!   class-like in an analyzed file. The host scans that file (and the
//!   classes it depends on, resolved through the project's composer
//!   autoloader) with Psalm's own scanner, so handlers see real
//!   `ClassLikeStorage`. Issues, `suppressed_issues` additions and recorded
//!   class/method references flow back to mir.
//!
//! The host boots a real Psalm `Codebase` with Psalm's internal stubs, so
//! plugins can use `UnionTypeComparator`, `CodeLocation` and `IssueBuffer`;
//! issues raised through `IssueBuffer` surface as mir plugin issues. Classes
//! named in types the host builds are scanned on demand.
//!
//! Other hook registrations (`AfterExpressionAnalysis`, taint hooks, …) are
//! reported in [`PsalmBridgePlugin::warnings`] and skipped — they would need
//! per-node RPC and a full Psalm `Codebase` shim.

use std::io::{BufRead, BufReader, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use parking_lot::Mutex;
use rustc_hash::FxHashMap;

use crate::{
    AfterClassLikeAnalysisEvent, AfterFunctionLikeAnalysisEvent, FunctionReturnTypeProviderEvent,
    HookFlags, MethodReturnTypeProviderEvent, MirPlugin, PluginIssue, ProvidedType, Severity,
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
    /// Hook files (`<plugin filename>`): required by the host, and the first
    /// class each declares is registered as an event-handler class.
    pub plugin_files: Vec<PathBuf>,
    /// `autoload.php` of a Psalm install outside the project's own vendor dir.
    /// When `None`, the host discovers one under the project root if the
    /// project's autoloader does not provide Psalm.
    pub psalm_autoload: Option<PathBuf>,
}

impl BridgeOptions {
    pub fn new(project_root: impl Into<PathBuf>, plugins: Vec<PsalmPluginSpec>) -> Self {
        Self {
            php_binary: "php".to_string(),
            project_root: project_root.into(),
            host_script_dir: None,
            plugins,
            plugin_files: Vec::new(),
            psalm_autoload: None,
        }
    }
}

/// Spawn one PHP host and run its `init`, loading every configured plugin.
fn start_host(
    options: &BridgeOptions,
    script: &Path,
) -> Result<(Rpc, serde_json::Value), BridgeError> {
    let mut child = Command::new(&options.php_binary)
        .arg("-d")
        .arg("display_errors=stderr")
        .arg(script)
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
            "pluginFiles": options.plugin_files,
            "psalmAutoload": options.psalm_autoload,
        }),
    )?;
    Ok((rpc, init))
}

/// How many hosts share class-like scanning: `MIR_PSALM_HOSTS`, else up to 4
/// bounded by the CPU count. Each holds its own Psalm codebase in memory.
fn class_host_count() -> usize {
    std::env::var("MIR_PSALM_HOSTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n >= 1)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .min(4)
        })
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
    /// Additional hosts, spawned on first use, that share the class-like
    /// scanning load: each scans files with its own Psalm codebase.
    extra_hosts: Vec<Mutex<Option<Rpc>>>,
    next_host: AtomicUsize,
    options: BridgeOptions,
    script: PathBuf,
    /// Set after an unrecoverable RPC failure; all further queries return
    /// `None` so analysis degrades to normal inference instead of erroring
    /// on every call site.
    dead: AtomicBool,
    stubs: Vec<PathBuf>,
    function_ids: Vec<String>,
    method_classes: Vec<String>,
    after_function_like: bool,
    after_class_like: bool,
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
        let (rpc, init) = start_host(options, &script)?;

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

        let extra_hosts = (1..class_host_count()).map(|_| Mutex::new(None)).collect();
        Ok(Self {
            rpc: Mutex::new(rpc),
            extra_hosts,
            next_host: AtomicUsize::new(0),
            options: options.clone(),
            script,
            dead: AtomicBool::new(false),
            stubs: str_list("stubs").into_iter().map(PathBuf::from).collect(),
            function_ids: str_list("functionIds"),
            method_classes: str_list("methodClasses"),
            after_function_like: init
                .get("afterFunctionLike")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            after_class_like: init
                .get("afterClassLike")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            warnings: str_list("warnings"),
            cache: Mutex::new(FxHashMap::default()),
        })
    }

    /// Whether the plugins registered anything mir can actually use.
    pub fn is_effectively_empty(&self) -> bool {
        self.stubs.is_empty()
            && self.function_ids.is_empty()
            && self.method_classes.is_empty()
            && !self.after_function_like
            && !self.after_class_like
    }

    /// Run `method` on the host. Returns the provided type string plus any
    /// issues the plugin raised; results that raised issues are never cached,
    /// since the issues belong to one call site.
    fn query_type(
        &self,
        method: &str,
        cache_key: String,
        params: serde_json::Value,
    ) -> (Option<ProvidedType>, Vec<PluginIssue>) {
        if self.dead.load(Ordering::Relaxed) {
            return (None, Vec::new());
        }
        if let Some(cached) = self.cache.lock().get(&cache_key) {
            return (cached.clone().map(ProvidedType::Parse), Vec::new());
        }
        let value = match self.call_host(method, params) {
            Some(value) => value,
            None => return (None, Vec::new()),
        };
        let type_string = value
            .get("type")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        let issues = parse_issues(&value);
        if issues.is_empty() {
            self.cache.lock().insert(cache_key, type_string.clone());
        }
        (type_string.map(ProvidedType::Parse), issues)
    }

    /// Run a class-like call on whichever host is idle, spawning extra hosts
    /// on demand; waits on a rotating host when all are busy.
    fn call_class_host(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Option<serde_json::Value> {
        if self.dead.load(Ordering::Relaxed) {
            return None;
        }
        if let Some(mut primary) = self.rpc.try_lock() {
            return self.finish_host_call(primary.call(method, params));
        }
        for slot in &self.extra_hosts {
            if let Some(guard) = slot.try_lock() {
                return self.call_extra_host(guard, method, params);
            }
        }
        let n = self.extra_hosts.len() + 1;
        let i = self.next_host.fetch_add(1, Ordering::Relaxed) % n;
        if i == 0 {
            let result = self.rpc.lock().call(method, params);
            self.finish_host_call(result)
        } else {
            let guard = self.extra_hosts[i - 1].lock();
            self.call_extra_host(guard, method, params)
        }
    }

    fn call_extra_host(
        &self,
        mut guard: parking_lot::MutexGuard<'_, Option<Rpc>>,
        method: &str,
        params: serde_json::Value,
    ) -> Option<serde_json::Value> {
        if guard.is_none() {
            match start_host(&self.options, &self.script) {
                Ok((rpc, _)) => *guard = Some(rpc),
                Err(e) => return self.finish_host_call(Err(e)),
            }
        }
        let rpc = guard.as_mut().expect("host started above");
        self.finish_host_call(rpc.call(method, params))
    }

    fn finish_host_call(
        &self,
        result: Result<serde_json::Value, BridgeError>,
    ) -> Option<serde_json::Value> {
        match result {
            Ok(value) => Some(value),
            Err(e) => {
                if !self.dead.swap(true, Ordering::Relaxed) {
                    eprintln!("mir: psalm plugin bridge disabled after error: {e}");
                }
                None
            }
        }
    }

    /// RPC call that disables the bridge on failure so analysis degrades to
    /// normal inference.
    fn call_host(&self, method: &str, params: serde_json::Value) -> Option<serde_json::Value> {
        if self.dead.load(Ordering::Relaxed) {
            return None;
        }
        match self.rpc.lock().call(method, params) {
            Ok(value) => Some(value),
            Err(e) => {
                if !self.dead.swap(true, Ordering::Relaxed) {
                    eprintln!("mir: psalm plugin bridge disabled after error: {e}");
                }
                None
            }
        }
    }
}

fn parse_issues(value: &serde_json::Value) -> Vec<PluginIssue> {
    let Some(list) = value.get("issues").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|raw| {
            let name = raw.get("name")?.as_str()?;
            let message = raw.get("message")?.as_str()?;
            let severity = match raw.get("severity").and_then(|v| v.as_str()) {
                Some("info") => Severity::Info,
                Some("warning") => Severity::Warning,
                _ => Severity::Error,
            };
            let mut issue = PluginIssue::new(name, message).with_severity(severity);
            let offset = |key: &str| raw.get(key).and_then(|v| v.as_u64()).map(|n| n as u32);
            if let (Some(start), Some(end)) = (offset("spanStart"), offset("spanEnd")) {
                issue = issue.with_span(php_ast::Span::new(start, end));
            }
            Some(issue)
        })
        .collect()
}

fn type_strings(types: &[crate::Type]) -> Vec<String> {
    types.iter().map(|t| t.to_string()).collect()
}

fn signature_key(head: &str, calling_class: Option<&str>, arg_types: &[crate::Type]) -> String {
    let mut key = String::from(head);
    key.push('\u{1e}');
    key.push_str(calling_class.unwrap_or(""));
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

    fn hooks(&self) -> HookFlags {
        HookFlags {
            after_function_like_analysis: self.after_function_like,
            after_class_like_analysis: self.after_class_like,
            ..HookFlags::default()
        }
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
        let (provided, issues) = self.query_type(
            "functionReturnType",
            signature_key(event.function_id, event.calling_class, event.arg_types),
            serde_json::json!({
                "functionId": event.function_id,
                "argTypes": type_strings(event.arg_types),
                "snippet": event.call_snippet,
                "file": event.file,
                "callingClass": event.calling_class,
                "spanStart": event.span.start,
                "spanEnd": event.span.end,
            }),
        );
        event.issues.borrow_mut().extend(issues);
        provided
    }

    fn method_return_type_classes(&self) -> Vec<String> {
        self.method_classes.clone()
    }

    fn method_return_type(
        &self,
        event: &MethodReturnTypeProviderEvent<'_>,
    ) -> Option<ProvidedType> {
        let (provided, issues) = self.query_type(
            "methodReturnType",
            signature_key(
                &format!("{}::{}", event.fqcn, event.method_name),
                event.calling_class,
                event.arg_types,
            ),
            serde_json::json!({
                "fqcn": event.fqcn,
                "methodName": event.method_name,
                "argTypes": type_strings(event.arg_types),
                "snippet": event.call_snippet,
                "file": event.file,
                "callingClass": event.calling_class,
                "spanStart": event.span.start,
                "spanEnd": event.span.end,
            }),
        );
        event.issues.borrow_mut().extend(issues);
        provided
    }

    fn after_class_like_analysis(&self, event: &mut AfterClassLikeAnalysisEvent<'_>) {
        let result = self.call_class_host(
            "afterClassLike",
            serde_json::json!({
                "fqcn": event.fqcn,
                "file": event.file,
                "spanStart": event.span.start,
                "spanEnd": event.span.end,
            }),
        );
        let Some(result) = result else { return };
        event.issues.extend(parse_issues(&result));
        let strings = |key: &str| -> Vec<String> {
            result
                .get(key)
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        event.suppressed_issues.extend(strings("suppressedIssues"));
        event.used_classes.extend(strings("usedClasses"));
        event
            .used_methods
            .extend(strings("usedMethods").into_iter().filter_map(|id| {
                id.split_once("::")
                    .map(|(c, m)| (c.to_string(), m.to_string()))
            }));
    }

    fn after_function_like_analysis(&self, event: &mut AfterFunctionLikeAnalysisEvent<'_>) {
        let params: Vec<serde_json::Value> = event
            .params
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.name,
                    "declaredType": p.declared_type,
                    "attributes": p.attributes.iter().map(|a| serde_json::json!({
                        "class": a.fq_class_name,
                        "spanStart": a.span.start,
                        "spanEnd": a.span.end,
                        "args": a.args.iter().map(|arg| serde_json::json!({
                            "name": arg.name,
                            "type": arg.type_string,
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                })
            })
            .collect();
        let result = self.call_host(
            "afterFunctionLike",
            serde_json::json!({
                "name": event.name,
                "class": event.class,
                "params": params,
                "snippet": event.snippet,
                "file": event.file,
                "spanStart": event.span.start,
                "spanEnd": event.span.end,
            }),
        );
        if let Some(result) = result {
            event.issues.extend(parse_issues(&result));
        }
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
        static NEXT_TMP: AtomicU64 = AtomicU64::new(0);
        let tmp = dir.join(format!(
            "mir-psalm-host-{digest}.php.tmp.{}.{}",
            std::process::id(),
            NEXT_TMP.fetch_add(1, Ordering::Relaxed)
        ));
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
