use wasm_bindgen::prelude::*;

use mir_analyzer::{PhpVersion, ProjectAnalyzer};
use serde::Serialize;

#[derive(Serialize)]
struct WasmIssue {
    name: &'static str,
    message: String,
    severity: String,
    line: u32,
    line_end: u32,
    col_start: u16,
    col_end: u16,
    snippet: Option<String>,
}

/// Stateful playground analyzer. Stubs are loaded once on construction and
/// reused across `analyze` calls. Re-initialized when the PHP version changes.
#[wasm_bindgen]
pub struct Playground {
    analyzer: ProjectAnalyzer,
    php_version: PhpVersion,
}

impl Default for Playground {
    fn default() -> Self {
        let version = PhpVersion::LATEST;
        let analyzer = make_analyzer(version);
        Self {
            analyzer,
            php_version: version,
        }
    }
}

#[wasm_bindgen]
impl Playground {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze a PHP source string and return a JSON array of issues.
    /// `php_version` is a string like `"8.3"` — falls back to latest if unparseable.
    pub fn analyze(&mut self, source: &str, php_version: &str) -> String {
        let version = php_version
            .parse::<PhpVersion>()
            .unwrap_or(PhpVersion::LATEST);
        if version != self.php_version {
            self.analyzer = make_analyzer(version);
            self.php_version = version;
        }
        let result = self.analyzer.re_analyze_file("<playground>", source);
        let issues: Vec<WasmIssue> = result
            .issues
            .iter()
            .filter(|i| !i.suppressed)
            .map(|i| WasmIssue {
                name: i.kind.name(),
                message: i.kind.message(),
                severity: i.severity.to_string(),
                line: i.location.line,
                line_end: i.location.line_end,
                col_start: i.location.col_start,
                col_end: i.location.col_end,
                snippet: i.snippet.clone(),
            })
            .collect();
        serde_json::to_string(&issues).unwrap_or_default()
    }
}

fn make_analyzer(version: PhpVersion) -> ProjectAnalyzer {
    let analyzer = ProjectAnalyzer::new().with_php_version(version);
    analyzer.load_stubs();
    analyzer
}
