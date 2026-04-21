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

/// Parsed contents of `mir.xml`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Source directories to analyze (from `<projectFiles>`).
    pub project_dirs: Vec<String>,
    /// Directories/files to skip (from `<ignoreFiles>`).
    pub ignore_dirs: Vec<String>,
    /// Per-issue-kind severity overrides from `<issueHandlers>`.
    pub issue_handlers: HashMap<String, ErrorLevel>,
    /// Global error level 1–8 (lower = stricter). 1 = errors only, 2 = +warnings, 3+ = +info.
    pub error_level: u8,
    /// Target PHP version string (e.g. `"8.2"`).
    pub php_version: Option<String>,
    /// Whether dead-code detection is enabled.
    pub find_unused_code: bool,
    /// Whether unused-variable checking is enabled.
    pub find_unused_variables: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_dirs: Vec::new(),
            ignore_dirs: Vec::new(),
            issue_handlers: HashMap::new(),
            error_level: 2,
            php_version: None,
            find_unused_code: false,
            find_unused_variables: false,
        }
    }
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
    use quick_xml::Reader;

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

                // Issue handler: <SomeIssueKind errorLevel="..." />  inside <issueHandlers>
                if path
                    .last()
                    .map(|s: &String| s == "issueHandlers")
                    .unwrap_or(false)
                {
                    for attr in e.attributes().flatten() {
                        if bytes_to_string(attr.key.as_ref()) == "errorLevel" {
                            if let Some(level) = ErrorLevel::from_str(&bytes_to_string(&attr.value))
                            {
                                config.issue_handlers.insert(name.clone(), level);
                            }
                        }
                    }
                }

                // <directory name="..."> inside <projectFiles> or <ignoreFiles>
                if name == "directory" {
                    collect_directory(&e, &path, &mut config);
                }

                text_buf.clear();
                path.push(name);
            }

            // Self-closing elements like <UndefinedVariable errorLevel="suppress" />
            Ok(Event::Empty(e)) => {
                let name = bytes_to_string(e.name().as_ref());

                if path
                    .last()
                    .map(|s: &String| s == "issueHandlers")
                    .unwrap_or(false)
                {
                    for attr in e.attributes().flatten() {
                        if bytes_to_string(attr.key.as_ref()) == "errorLevel" {
                            if let Some(level) = ErrorLevel::from_str(&bytes_to_string(&attr.value))
                            {
                                config.issue_handlers.insert(name.clone(), level);
                            }
                        }
                    }
                }

                if name == "directory" {
                    collect_directory(&e, &path, &mut config);
                }
            }

            Ok(Event::Text(t)) => {
                text_buf = t
                    .xml_content()
                    .map_err(|e| ConfigError::Parse(e.to_string()))?
                    .to_string();
            }

            Ok(Event::End(_)) => {
                let key = path.pop().unwrap_or_default();
                let parent = path.last().map(|s| s.as_str()).unwrap_or("");
                match (key.as_str(), parent) {
                    ("phpVersion", _) if !text_buf.is_empty() => {
                        config.php_version = Some(text_buf.clone());
                    }
                    ("errorLevel", "mir") => {
                        if let Ok(n) = text_buf.parse::<u8>() {
                            config.error_level = n.clamp(1, 8);
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

/// Extract `name` attribute from a `<directory name="..."/>` element and push to the
/// right list based on the current element path.
fn collect_directory<'a>(
    e: &quick_xml::events::BytesStart<'a>,
    path: &[String],
    config: &mut Config,
) {
    let parent = path.last().map(|s| s.as_str()).unwrap_or("");
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

fn bytes_to_string(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
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
        if let Some(by_kind) = self.entries.get_mut(file) {
            if let Some(snippets) = by_kind.get_mut(issue_kind) {
                if let Some(pos) = snippets.iter().position(|s| s == snippet) {
                    snippets.remove(pos);
                    return true;
                }
            }
        }
        false
    }

    /// Return true if the (file, issue_kind) pair exists in the baseline
    /// regardless of snippet.  Used as a fallback when no snippet is available.
    #[allow(dead_code)]
    pub fn contains_kind(&self, file: &str, issue_kind: &str) -> bool {
        self.entries
            .get(file)
            .and_then(|m| m.get(issue_kind))
            .map(|v| !v.is_empty())
            .unwrap_or(false)
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
                out.push_str(&format!("    <{}>\n", kind));
                for snippet in snippets {
                    out.push_str(&format!("      <code><![CDATA[{}]]></code>\n", snippet));
                }
                out.push_str(&format!("    </{}>\n", kind));
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
    use quick_xml::Reader;

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
                    _ if path.last().map(|s: &String| s == "file").unwrap_or(false) => {
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
                text_buf = String::from_utf8_lossy(cd.as_ref()).trim().to_string();
            }
            Ok(Event::Text(t)) => {
                let s = t
                    .xml_content()
                    .map_err(|e| ConfigError::Parse(e.to_string()))?;
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
