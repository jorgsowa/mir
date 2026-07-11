use owo_colors::OwoColorize;

use mir_issues::{Issue, Severity};

/// Renders an issue the way `mir`'s default text output prints it, e.g.
/// `src/foo.php:3:1 error[MIR0002] UndefinedVariable: ...`.
pub fn format_issue(issue: &Issue) -> String {
    let sev = match issue.severity {
        Severity::Error => "error".red().to_string(),
        Severity::Warning => "warning".yellow().to_string(),
        Severity::Info => "info".blue().to_string(),
    };
    format!(
        "{} {}[{}] {}: {}",
        issue.location.bright_black(),
        sev,
        issue.kind.code().bright_black(),
        issue.kind.name().bold(),
        issue.kind.message()
    )
}

pub fn format_junit(issues: &[&Issue]) -> String {
    use std::collections::HashMap;

    let mut by_file: HashMap<&str, Vec<&Issue>> = HashMap::new();
    for issue in issues {
        by_file
            .entry(issue.location.file.as_ref())
            .or_default()
            .push(issue);
    }

    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let total_failures: usize = issues
        .iter()
        .filter(|i| i.severity == Severity::Error)
        .count();
    out.push_str(&format!(
        "<testsuites name=\"mir\" tests=\"{}\" failures=\"{}\">\n",
        issues.len(),
        total_failures,
    ));

    let mut files: Vec<&str> = by_file.keys().copied().collect();
    files.sort_unstable();

    for file in files {
        let file_issues = &by_file[file];
        let failures = file_issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        out.push_str(&format!(
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\">\n",
            xml_escape(file),
            file_issues.len(),
            failures,
        ));
        for issue in file_issues.iter() {
            let name = issue.kind.name();
            let msg = issue.kind.message();
            let severity = match issue.severity {
                Severity::Error => "failure",
                Severity::Warning => "warning",
                Severity::Info => "notice",
            };
            out.push_str(&format!(
                "    <testcase name=\"{}\" classname=\"{}\">\n",
                xml_escape(name),
                xml_escape(file),
            ));
            out.push_str(&format!(
                "      <{} message=\"{}\" type=\"{}\">{}</{}>\n",
                severity,
                xml_escape(&msg),
                xml_escape(name),
                xml_escape(&format!(
                    "{}:{}:{} {} {}: {}",
                    file, issue.location.line, issue.location.col_start, issue.severity, name, msg
                )),
                severity,
            ));
            out.push_str("    </testcase>\n");
        }
        out.push_str("  </testsuite>\n");
    }

    out.push_str("</testsuites>\n");
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// FNV-1a 64-bit hash for stable partial fingerprints without extra dependencies.
fn fnv1a(data: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in data.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x00000100000001b3);
    }
    hash
}

pub fn format_sarif(issues: &[&Issue]) -> String {
    let mut rule_map: std::collections::HashMap<String, Severity> =
        std::collections::HashMap::new();
    for issue in issues {
        rule_map
            .entry(issue.kind.name().to_string())
            .or_insert_with(|| issue.kind.default_severity());
    }
    let mut rule_ids: Vec<String> = rule_map.keys().cloned().collect();
    rule_ids.sort_unstable();

    let rules_json: Vec<serde_json::Value> = rule_ids
        .iter()
        .map(|id| {
            let level = match rule_map[id] {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };
            let tag = if id.starts_with("Tainted") {
                "security"
            } else {
                "maintainability"
            };
            serde_json::json!({
                "id": id,
                "name": id,
                "shortDescription": { "text": id },
                "helpUri": "https://github.com/jorgsowa/mir",
                "defaultConfiguration": { "level": level },
                "properties": { "tags": [tag] },
            })
        })
        .collect();

    let results_json: Vec<serde_json::Value> = issues
        .iter()
        .map(|issue| {
            let level = match issue.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "note",
            };

            // Fingerprint based on issue kind + snippet content (not location) so
            // GitHub Code Scanning can track findings across renames/reformats.
            let fingerprint_input = format!(
                "{}:{}",
                issue.kind.name(),
                issue.snippet.as_deref().unwrap_or("")
            );
            let fingerprint = format!("{:016x}", fnv1a(&fingerprint_input));

            // rank: Error → 90, Warning → 95, Info → 99 (matches Psalm's 90–99 range).
            let rank = match issue.severity {
                Severity::Error => 90.0_f64,
                Severity::Warning => 95.0,
                Severity::Info => 99.0,
            };

            // SARIF 2.1.0 §3.30.5: columns are 1-based; col_start/col_end are 0-based.
            serde_json::json!({
                "ruleId": issue.kind.name(),
                "level": level,
                "rank": rank,
                "message": { "text": issue.kind.message() },
                "partialFingerprints": {
                    "primaryLocationLineHash": fingerprint,
                },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": issue.location.file.as_ref(),
                            "uriBaseId": "%SRCROOT%",
                        },
                        "region": {
                            "startLine": issue.location.line,
                            "endLine": issue.location.line_end,
                            "startColumn": issue.location.col_start + 1,
                            "endColumn": issue.location.col_end + 1,
                        }
                    }
                }]
            })
        })
        .collect();

    let sarif = serde_json::json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "mir",
                    "informationUri": "https://github.com/jorgsowa/mir",
                    "rules": rules_json,
                }
            },
            "results": results_json,
        }]
    });

    serde_json::to_string_pretty(&sarif).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use mir_issues::{Issue, IssueKind, Location};

    use super::*;

    #[test]
    fn format_issue_includes_code() {
        let issue = Issue::new(
            IssueKind::UndefinedClass {
                name: "Foo".to_string(),
            },
            Location {
                file: Arc::from("src/x.php"),
                line: 1,
                line_end: 1,
                col_start: 0,
                col_end: 3,
            },
        );
        // Strip ANSI escape sequences so the assertion isn't dependent on
        // owo-colors' tty detection.
        let raw = format_issue(&issue);
        let stripped: String = {
            let mut out = String::new();
            let mut chars = raw.chars();
            while let Some(c) = chars.next() {
                if c == '\u{1b}' {
                    for c2 in chars.by_ref() {
                        if c2 == 'm' {
                            break;
                        }
                    }
                } else {
                    out.push(c);
                }
            }
            out
        };
        assert!(
            stripped.contains("error[MIR0005] UndefinedClass:"),
            "format_issue output missing code/name segment: {stripped:?}",
        );
    }
}
