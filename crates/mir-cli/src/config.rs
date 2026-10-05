/// Project-level configuration parsed from `mir.xml`.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Per-issue severity override from `<issueHandlers>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    Error,
    Warning,
    Info,
    Suppress,
}

impl ErrorLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "error" => Some(Self::Error),
            "warning" | "warn" => Some(Self::Warning),
            "info" | "notice" => Some(Self::Info),
            "suppress" | "none" => Some(Self::Suppress),
            _ => None,
        }
    }
}

/// One `<pluginClass class="..."/>` entry from `<plugins>` — a Psalm PHP
/// plugin entry-point class, exactly as psalm.xml declares them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PsalmPluginEntry {
    pub class: String,
    /// The `<pluginClass>` element re-wrapped around its inner XML, when the
    /// entry carried plugin-specific config; passed to the plugin as its
    /// `SimpleXMLElement`.
    pub config_xml: Option<String>,
}

/// Parsed contents of `mir.xml`.
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Source directories to analyze (from `<projectFiles>`).
    pub project_dirs: Vec<String>,
    /// Directories/files to skip (from `<ignoreFiles>`).
    pub ignore_dirs: Vec<String>,
    /// Extensions treated as PHP source (from `<fileExtensions><extension name=".inc"/>`).
    /// Empty means `.php` only; `php` must be listed explicitly once any are given.
    pub file_extensions: Vec<String>,
    /// Per-issue-kind severity overrides from `<issueHandlers>`.
    pub issue_handlers: HashMap<String, ErrorLevel>,
    /// Optional Psalm-compatible strictness, 1 (strictest) to 8 (most lenient).
    /// `None` keeps mir's own severities.
    pub error_level: Option<u8>,
    /// Target PHP version string (e.g. `"8.2"`). Accepts both root attribute and child element.
    pub php_version: Option<String>,
    /// Whether dead-code detection is enabled.
    pub find_unused_code: bool,
    /// Whether unused-variable checking is enabled.
    pub find_unused_variables: bool,
    /// Psalm's `memoizeMethodCallResults` root attribute.
    pub memoize_method_call_results: bool,
    /// External stub files to load (from `<stubs><file name="..."/>`).
    pub stub_files: Vec<String>,
    /// External stub directories to load (from `<stubs><directory name="..."/>`).
    pub stub_dirs: Vec<String>,
    /// Function names from `<forbiddenFunctions><function name="..."/>`.
    pub forbidden_functions: Vec<String>,
    /// Psalm PHP plugins (from `<plugins><pluginClass class="..."/>`).
    pub psalm_plugins: Vec<PsalmPluginEntry>,
    /// File-based Psalm plugins (from `<plugins><plugin filename="..."/>`).
    pub psalm_plugin_files: Vec<String>,
    /// Rust cdylib plugins (from `<plugins><rustPlugin path="..."/>`).
    pub rust_plugins: Vec<String>,
}

/// Errors that can occur when loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read config file: {0}")]
    Io(String),
    #[error("XML parse error: {0}")]
    Parse(String),
}

// ---------------------------------------------------------------------------
// Config impl
// ---------------------------------------------------------------------------

impl Config {
    /// Walk from `start_dir` upward looking for `mir.xml` (or `psalm.xml` as a compatibility fallback).
    /// Returns the path if found.
    pub fn find(start_dir: &Path) -> Option<PathBuf> {
        let mut dir = start_dir.to_path_buf();
        loop {
            let mir = dir.join("mir.xml");
            if mir.exists() {
                return Some(mir);
            }
            let psalm = dir.join("psalm.xml");
            if psalm.exists() {
                return Some(psalm);
            }
            if !dir.pop() {
                return None;
            }
        }
    }

    /// Load and parse `mir.xml` at the given path.
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let xml = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
        Self::parse(&xml)
    }

    /// Parse `mir.xml` from a string.
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        parse_xml(xml)
    }
}

// ---------------------------------------------------------------------------
// XML parser (quick-xml event API)
// ---------------------------------------------------------------------------

fn parse_xml(xml: &str) -> Result<Config, ConfigError> {
    use quick_xml::events::Event;
    use quick_xml::{Reader, XmlVersion};

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut config = Config::default();
    // Element path stack, e.g. ["mir", "projectFiles"]
    let mut path: Vec<String> = Vec::new();
    // Accumulated text content for the current element
    let mut text_buf = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = bytes_to_string(e.name().as_ref());

                // phpVersion as attribute on the root element: <mir phpVersion="8.2">
                if path.is_empty() && (name == "mir" || name == "psalm") {
                    for attr in e.attributes().flatten() {
                        if bytes_to_string(attr.key.as_ref()) == "phpVersion"
                            && config.php_version.is_none()
                        {
                            let val = bytes_to_string(&attr.value);
                            if !val.is_empty() {
                                config.php_version = Some(val);
                            }
                        }
                    }
                }

                if path.is_empty() {
                    if let Some(level) = attr_value(&e, "errorLevel").and_then(parse_psalm_level) {
                        config.error_level = Some(level);
                    }
                    config.memoize_method_call_results = matches!(
                        attr_value(&e, "memoizeMethodCallResults").as_deref(),
                        Some("true" | "1")
                    );
                    config.find_unused_code |= is_true_attr(&e, "findUnusedCode");
                    config.find_unused_variables |= is_true_attr(&e, "findUnusedVariables");
                }

                // Issue handler: <SomeIssueKind errorLevel="..." />  inside <issueHandlers>
                if path.last().is_some_and(|s: &String| s == "issueHandlers") {
                    for attr in e.attributes().flatten() {
                        if bytes_to_string(attr.key.as_ref()) == "errorLevel" {
                            if let Some(level) = ErrorLevel::from_str(&bytes_to_string(&attr.value))
                            {
                                config.issue_handlers.insert(name.clone(), level);
                            }
                        }
                    }
                }

                // <file name="..."> or <directory name="..."> inside project files.
                if name == "file" || name == "directory" {
                    collect_project_file_entry(&e, &path, &mut config);
                }
                collect_file_extension_entry(&e, &path, &mut config);

                // <file name="..."> or <directory name="..."> inside <stubs>
                if name == "file" || name == "directory" {
                    collect_stub_entry(&e, &path, &mut config);
                }
                collect_forbidden_function_entry(&e, &path, &mut config);

                // <pluginClass class="..."> inside <plugins>: consume the whole
                // element (its inner XML is the plugin's own config, passed
                // through verbatim) — so it is NOT pushed onto the path.
                if name == "pluginClass" && path.last().is_some_and(|s: &String| s == "plugins") {
                    let class = attr_value(&e, "class");
                    let inner = reader
                        .read_text(e.name())
                        .map_err(|e| ConfigError::Parse(e.to_string()))?;
                    if let Some(class) = class {
                        // Raw inner XML (markup included) — passed through to
                        // the plugin verbatim as its config element body.
                        let inner = inner.trim();
                        config.psalm_plugins.push(PsalmPluginEntry {
                            class,
                            config_xml: (!inner.is_empty())
                                .then(|| format!("<pluginClass>{inner}</pluginClass>")),
                        });
                    }
                    continue;
                }

                // <rustPlugin path="..."> inside <plugins>
                if name == "rustPlugin" && path.last().is_some_and(|s: &String| s == "plugins") {
                    if let Some(p) = attr_value(&e, "path") {
                        config.rust_plugins.push(p);
                    }
                }

                if name == "plugin" && path.last().is_some_and(|s: &String| s == "plugins") {
                    if let Some(f) = attr_value(&e, "filename") {
                        config.psalm_plugin_files.push(f);
                    }
                }

                text_buf.clear();
                path.push(name);
            }

            // Self-closing elements like <UndefinedVariable errorLevel="suppress" />
            Ok(Event::Empty(e)) => {
                let name = bytes_to_string(e.name().as_ref());

                if path.last().is_some_and(|s: &String| s == "issueHandlers") {
                    for attr in e.attributes().flatten() {
                        if bytes_to_string(attr.key.as_ref()) == "errorLevel" {
                            if let Some(level) = ErrorLevel::from_str(&bytes_to_string(&attr.value))
                            {
                                config.issue_handlers.insert(name.clone(), level);
                            }
                        }
                    }
                }

                if name == "file" || name == "directory" {
                    collect_project_file_entry(&e, &path, &mut config);
                }
                collect_file_extension_entry(&e, &path, &mut config);

                // <file name="..."/> or <directory name="..."/> inside <stubs>
                if name == "file" || name == "directory" {
                    collect_stub_entry(&e, &path, &mut config);
                }
                collect_forbidden_function_entry(&e, &path, &mut config);

                if path.last().is_some_and(|s: &String| s == "plugins") {
                    if name == "pluginClass" {
                        if let Some(class) = attr_value(&e, "class") {
                            config.psalm_plugins.push(PsalmPluginEntry {
                                class,
                                config_xml: None,
                            });
                        }
                    } else if name == "plugin" {
                        if let Some(f) = attr_value(&e, "filename") {
                            config.psalm_plugin_files.push(f);
                        }
                    } else if name == "rustPlugin" {
                        if let Some(p) = attr_value(&e, "path") {
                            config.rust_plugins.push(p);
                        }
                    }
                }
            }

            Ok(Event::Text(t)) => {
                text_buf = t.xml_content(XmlVersion::Implicit1_0).to_string();
            }

            Ok(Event::End(_)) => {
                let key = path.pop().unwrap_or_default();
                let parent = path.last().map_or("", |s| s.as_str());
                match (key.as_str(), parent) {
                    ("phpVersion", _) if !text_buf.is_empty() => {
                        config.php_version = Some(text_buf.clone());
                    }
                    ("errorLevel", "mir") => {
                        if let Some(level) = parse_psalm_level(text_buf.clone()) {
                            config.error_level = Some(level);
                        }
                    }
                    ("findUnusedCode", _) => {
                        config.find_unused_code = text_buf == "true";
                    }
                    ("findUnusedVariables", _) => {
                        config.find_unused_variables = text_buf == "true";
                    }
                    _ => {}
                }
                text_buf.clear();
            }

            Ok(Event::Eof) => break,
            Err(e) => return Err(ConfigError::Parse(e.to_string())),
            _ => {}
        }
    }

    Ok(config)
}

/// Extract a project or ignored file path from its `name` attribute.
fn collect_project_file_entry<'a>(
    e: &quick_xml::events::BytesStart<'a>,
    path: &[String],
    config: &mut Config,
) {
    let parent = path.last().map_or("", |s| s.as_str());
    for attr in e.attributes().flatten() {
        if bytes_to_string(attr.key.as_ref()) == "name" {
            let val = bytes_to_string(&attr.value);
            match parent {
                "projectFiles" => config.project_dirs.push(val),
                "ignoreFiles" => config.ignore_dirs.push(val),
                _ => {}
            }
        }
    }
}

/// Handle `<extension name=".inc"/>` inside `<fileExtensions>`.
fn collect_file_extension_entry(
    e: &quick_xml::events::BytesStart<'_>,
    path: &[String],
    config: &mut Config,
) {
    if path.last().map_or("", |s| s.as_str()) == "fileExtensions"
        && bytes_to_string(e.name().as_ref()) == "extension"
    {
        config.file_extensions.extend(attr_value(e, "name"));
    }
}

/// Handle `<file name="..."/>` and `<directory name="..."/>` inside `<stubs>`.
fn collect_stub_entry<'a>(
    e: &quick_xml::events::BytesStart<'a>,
    path: &[String],
    config: &mut Config,
) {
    if path.last().map_or("", |s| s.as_str()) != "stubs" {
        return;
    }
    let elem = bytes_to_string(e.name().as_ref());
    for attr in e.attributes().flatten() {
        if bytes_to_string(attr.key.as_ref()) == "name" {
            let val = bytes_to_string(&attr.value);
            match elem.as_str() {
                "file" => config.stub_files.push(val),
                "directory" => config.stub_dirs.push(val),
                _ => {}
            }
        }
    }
}

/// Handle `<function name="..."/>` inside `<forbiddenFunctions>`.
fn collect_forbidden_function_entry(
    e: &quick_xml::events::BytesStart<'_>,
    path: &[String],
    config: &mut Config,
) {
    if path.last().is_some_and(|s| s == "forbiddenFunctions")
        && bytes_to_string(e.name().as_ref()) == "function"
    {
        config.forbidden_functions.extend(attr_value(e, "name"));
    }
}

fn bytes_to_string(s: impl AsRef<str>) -> String {
    s.as_ref().to_owned()
}

fn parse_psalm_level(s: String) -> Option<u8> {
    s.trim().parse::<u8>().ok().map(|n| n.clamp(1, 8))
}

/// Value of the named attribute on an element, if present.
fn attr_value(e: &quick_xml::events::BytesStart<'_>, name: &str) -> Option<String> {
    e.attributes().flatten().find_map(|attr| {
        (bytes_to_string(attr.key.as_ref()) == name).then(|| bytes_to_string(&attr.value))
    })
}

fn is_true_attr(e: &quick_xml::events::BytesStart<'_>, name: &str) -> bool {
    matches!(attr_value(e, name).as_deref(), Some("true" | "1"))
}

// ---------------------------------------------------------------------------
// Baseline
// ---------------------------------------------------------------------------

/// Parsed contents of a baseline XML (`baseline.xml` / `psalm-baseline.xml`).
///
/// Structure: `file_path → issue_kind → [code_snippets]`
///
/// A code snippet is the trimmed source text of the flagged expression — the
/// `<code>` element inside a baseline entry.  Matching is done by
/// (file, issue_kind, snippet) so that refactors that change line numbers
/// do not invalidate the baseline.
#[derive(Debug, Clone, Default)]
pub struct Baseline {
    /// Outer key: source-relative file path (e.g. `"application/server/Foo.php"`).
    /// Inner key: issue kind name (e.g. `"InvalidArgument"`).
    /// Value: sorted vec of code snippets to consume (each entry is used once).
    pub entries: HashMap<String, HashMap<String, Vec<String>>>,
}

impl Baseline {
    /// Load a baseline from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let xml = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e.to_string()))?;
        Self::parse(&xml)
    }

    /// Parse a baseline XML string.
    pub fn parse(xml: &str) -> Result<Self, ConfigError> {
        parse_baseline_xml(xml)
    }

    /// Return true if the given (file, issue_kind, snippet) triple is present
    /// in the baseline.  Each matching entry is consumed once so duplicate
    /// suppressions work correctly.
    pub fn consume(&mut self, file: &str, issue_kind: &str, snippet: &str) -> bool {
        let baseline_file = self
            .entries
            .contains_key(file)
            .then(|| file.to_owned())
            .or_else(|| {
                self.entries
                    .keys()
                    .find(|candidate| std::path::Path::new(candidate) == std::path::Path::new(file))
                    .cloned()
            });

        if let Some(by_kind) = baseline_file.and_then(|file| self.entries.get_mut(&file)) {
            if let Some(snippets) = by_kind.get_mut(issue_kind) {
                if let Some(pos) = snippets.iter().position(|s| s == snippet) {
                    snippets.remove(pos);
                    return true;
                }
            }
        }
        false
    }

    /// Serialize this baseline to a Psalm-compatible XML file.
    pub fn write(&self, path: &std::path::Path) -> Result<(), ConfigError> {
        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n");

        let mut files: Vec<&String> = self.entries.keys().collect();
        files.sort_unstable();

        for file in files {
            let by_kind = &self.entries[file];
            let mut kinds: Vec<&String> = by_kind.keys().collect();
            kinds.sort_unstable();

            out.push_str(&format!("  <file src=\"{}\">\n", xml_escape_attr(file)));
            for kind in kinds {
                let snippets = &by_kind[kind];
                out.push_str(&format!("    <{kind}>\n"));
                for snippet in snippets {
                    out.push_str(&format!("      <code><![CDATA[{snippet}]]></code>\n"));
                }
                out.push_str(&format!("    </{kind}>\n"));
            }
            out.push_str("  </file>\n");
        }

        out.push_str("</files>\n");

        std::fs::write(path, out).map_err(|e| ConfigError::Io(e.to_string()))
    }
}

fn xml_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn parse_baseline_xml(xml: &str) -> Result<Baseline, ConfigError> {
    use quick_xml::events::Event;
    use quick_xml::{Reader, XmlVersion};

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut baseline = Baseline::default();
    // Stack: ["files", "file", "IssuKind"]
    let mut path: Vec<String> = Vec::new();
    let mut current_file: Option<String> = None;
    let mut current_kind: Option<String> = None;
    let mut text_buf = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = bytes_to_string(e.name().as_ref());
                match name.as_str() {
                    "file" => {
                        for attr in e.attributes().flatten() {
                            if bytes_to_string(attr.key.as_ref()) == "src" {
                                current_file = Some(bytes_to_string(&attr.value));
                            }
                        }
                        current_kind = None;
                    }
                    "files" => {}
                    _ if path.last().is_some_and(|s: &String| s == "file") => {
                        // Direct child of <file> is an issue-kind element
                        current_kind = Some(name.clone());
                    }
                    _ => {}
                }
                text_buf.clear();
                path.push(name);
            }
            Ok(Event::Empty(e)) => {
                // Self-closing <file> or <code/> — handled below
                let name = bytes_to_string(e.name().as_ref());
                if name == "file" {
                    for attr in e.attributes().flatten() {
                        if bytes_to_string(attr.key.as_ref()) == "src" {
                            current_file = Some(bytes_to_string(&attr.value));
                        }
                    }
                }
            }
            Ok(Event::CData(cd)) => {
                text_buf = cd.as_ref().trim().to_string();
            }
            Ok(Event::Text(t)) => {
                let s = t.xml_content(XmlVersion::Implicit1_0);
                let trimmed = s.trim().to_string();
                if !trimmed.is_empty() {
                    text_buf = trimmed;
                }
            }
            Ok(Event::End(e)) => {
                let name = bytes_to_string(e.name().as_ref());
                match name.as_str() {
                    "code" => {
                        // Record this snippet
                        if let (Some(file), Some(kind)) = (&current_file, &current_kind) {
                            let snippet = std::mem::take(&mut text_buf);
                            baseline
                                .entries
                                .entry(file.clone())
                                .or_default()
                                .entry(kind.clone())
                                .or_default()
                                .push(snippet);
                        }
                    }
                    "file" => {
                        current_file = None;
                        current_kind = None;
                    }
                    _ if Some(&name) == current_kind.as_ref() => {
                        current_kind = None;
                    }
                    _ => {}
                }
                path.pop();
                text_buf.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(ConfigError::Parse(e.to_string())),
            _ => {}
        }
    }

    Ok(baseline)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_file_extensions() {
        let cfg = Config::parse(
            r#"<mir><fileExtensions><extension name=".php"/><extension name=".inc"></extension></fileExtensions></mir>"#,
        )
        .unwrap();
        assert_eq!(cfg.file_extensions, [".php", ".inc"]);
    }

    #[test]
    fn file_extensions_default_to_empty() {
        assert!(Config::parse("<mir></mir>")
            .unwrap()
            .file_extensions
            .is_empty());
    }

    #[test]
    fn parses_php_version_child_element() {
        let cfg = Config::parse(r#"<mir><phpVersion>8.1</phpVersion></mir>"#).unwrap();
        assert_eq!(cfg.php_version.as_deref(), Some("8.1"));
    }

    #[test]
    fn parses_php_version_root_attribute() {
        let cfg = Config::parse(r#"<mir phpVersion="8.2"></mir>"#).unwrap();
        assert_eq!(cfg.php_version.as_deref(), Some("8.2"));
    }

    #[test]
    fn parses_unused_code_root_attributes() {
        let cfg = Config::parse(r#"<psalm findUnusedCode="true" findUnusedVariables="1"></psalm>"#)
            .unwrap();
        assert!(cfg.find_unused_code && cfg.find_unused_variables);
        let cfg = Config::parse(r#"<psalm findUnusedCode="false"></psalm>"#).unwrap();
        assert!(!cfg.find_unused_code && !cfg.find_unused_variables);
    }

    #[test]
    fn parses_memoize_method_call_results_root_attribute() {
        let on = Config::parse(r#"<psalm memoizeMethodCallResults="true"></psalm>"#).unwrap();
        assert!(on.memoize_method_call_results);
        let off = Config::parse(r#"<psalm memoizeMethodCallResults="false"></psalm>"#).unwrap();
        assert!(!off.memoize_method_call_results);
        assert!(
            !Config::parse("<psalm></psalm>")
                .unwrap()
                .memoize_method_call_results
        );
    }

    #[test]
    fn root_attribute_does_not_override_cli_override() {
        // Simulate: config file has attribute, CLI flag would overwrite in main.rs.
        // The XML parser itself should accept the attribute form.
        let cfg = Config::parse(r#"<psalm phpVersion="7.4"></psalm>"#).unwrap();
        assert_eq!(cfg.php_version.as_deref(), Some("7.4"));
    }

    #[test]
    fn parses_forbidden_functions() {
        let cfg = Config::parse(
            r#"<psalm>
                <forbiddenFunctions>
                    <function name="var_dump"/>
                    <function name="shell_exec"/>
                </forbiddenFunctions>
            </psalm>"#,
        )
        .unwrap();
        assert_eq!(cfg.forbidden_functions, vec!["var_dump", "shell_exec"]);
        assert!(Config::parse("<psalm></psalm>")
            .unwrap()
            .forbidden_functions
            .is_empty());
    }

    #[test]
    fn parses_stubs_file_entries() {
        let cfg = Config::parse(
            r#"<mir>
                <stubs>
                    <file name="stubs/helpers.php"/>
                    <file name="stubs/ide.php"/>
                </stubs>
            </mir>"#,
        )
        .unwrap();
        assert_eq!(cfg.stub_files, vec!["stubs/helpers.php", "stubs/ide.php"]);
        assert!(cfg.stub_dirs.is_empty());
    }

    #[test]
    fn parses_stubs_directory_entries() {
        let cfg = Config::parse(
            r#"<mir>
                <stubs>
                    <directory name="stubs/doctrine"/>
                </stubs>
            </mir>"#,
        )
        .unwrap();
        assert_eq!(cfg.stub_dirs, vec!["stubs/doctrine"]);
        assert!(cfg.stub_files.is_empty());
    }

    #[test]
    fn parses_plugin_class_entries() {
        let cfg = Config::parse(
            r#"<psalm>
                <plugins>
                    <pluginClass class="Psalm\PhpUnitPlugin\Plugin"/>
                    <pluginClass class="Foo\Plugin">
                        <option value="1"/>
                    </pluginClass>
                </plugins>
            </psalm>"#,
        )
        .unwrap();
        assert_eq!(
            cfg.psalm_plugins,
            vec![
                PsalmPluginEntry {
                    class: "Psalm\\PhpUnitPlugin\\Plugin".to_string(),
                    config_xml: None,
                },
                PsalmPluginEntry {
                    class: "Foo\\Plugin".to_string(),
                    config_xml: Some(
                        "<pluginClass><option value=\"1\"/></pluginClass>".to_string()
                    ),
                },
            ]
        );
    }

    #[test]
    fn parses_plugin_filename_entries() {
        let cfg = Config::parse(
            r#"<psalm>
                <plugins>
                    <plugin filename="tools/Hooks.php"/>
                    <pluginClass class="Foo\Plugin"/>
                </plugins>
            </psalm>"#,
        )
        .unwrap();
        assert_eq!(cfg.psalm_plugin_files, vec!["tools/Hooks.php"]);
        assert_eq!(cfg.psalm_plugins.len(), 1);
    }

    #[test]
    fn parses_rust_plugin_entries() {
        let cfg = Config::parse(
            r#"<mir>
                <plugins>
                    <rustPlugin path="plugins/libmy_plugin.dylib"/>
                </plugins>
            </mir>"#,
        )
        .unwrap();
        assert_eq!(cfg.rust_plugins, vec!["plugins/libmy_plugin.dylib"]);
        assert!(cfg.psalm_plugins.is_empty());
    }

    #[test]
    fn plugin_class_with_config_still_parses_following_elements() {
        let cfg = Config::parse(
            r#"<mir>
                <plugins>
                    <pluginClass class="Foo\Plugin"><x/></pluginClass>
                </plugins>
                <projectFiles>
                    <directory name="src"/>
                </projectFiles>
            </mir>"#,
        )
        .unwrap();
        assert_eq!(cfg.psalm_plugins.len(), 1);
        assert_eq!(cfg.project_dirs, vec!["src"]);
    }

    #[test]
    fn stubs_directory_does_not_pollute_project_dirs() {
        let cfg = Config::parse(
            r#"<mir>
                <projectFiles>
                    <directory name="src"/>
                </projectFiles>
                <stubs>
                    <directory name="stubs/ext"/>
                </stubs>
            </mir>"#,
        )
        .unwrap();
        assert_eq!(cfg.project_dirs, vec!["src"]);
        assert_eq!(cfg.stub_dirs, vec!["stubs/ext"]);
    }

    #[test]
    fn baseline_consumes_equivalent_platform_path() {
        let mut baseline = Baseline::parse(
            r#"<files><file src="./src/Foo.php"><UndefinedVariable><code><![CDATA[$x]]></code></UndefinedVariable></file></files>"#,
        )
        .unwrap();

        let platform_path = std::path::Path::new(".").join("src").join("Foo.php");
        assert!(baseline.consume(platform_path.to_str().unwrap(), "UndefinedVariable", "$x"));
        assert!(baseline.entries["./src/Foo.php"]["UndefinedVariable"].is_empty());
    }
}
