//! Inline issue suppression via source comments.
//!
//! Lets users silence a single false positive without touching `mir.xml` or a
//! baseline file. A [`SuppressionMap`] is built once per file from its source
//! text and consulted as a final post-filter over the analyzer's issues
//! (`batch.rs`), so it applies uniformly across every emitting pass —
//! body analysis, the collector, class checks and dead-code detection.
//!
//! ## Recognised directives
//!
//! Native (preferred), matching the existing `@mir-check` convention:
//!
//! | Directive                  | Scope                                   |
//! |----------------------------|-----------------------------------------|
//! | `@mir-ignore [Kind …]`     | trailing comment → its line; otherwise the next code line |
//! | `@mir-ignore-line [Kind …]`      | the comment's own line            |
//! | `@mir-ignore-next-line [Kind …]` | the next physical line            |
//! | `@mir-ignore-file [Kind …]`      | the whole file                    |
//!
//! `@mir-suppress*` is accepted as an alias of `@mir-ignore*`.
//!
//! Third-party aliases for drop-in compatibility:
//!
//! | Directive                   | Scope / kinds                          |
//! |-----------------------------|----------------------------------------|
//! | `@psalm-suppress Kind …`    | like `@mir-ignore` (named kinds)       |
//! | `@suppress Kind …`          | like `@mir-ignore` (named kinds)       |
//! | `@phpstan-ignore-line`      | the comment's own line, all kinds      |
//! | `@phpstan-ignore-next-line` | the next line, all kinds               |
//! | `@phpstan-ignore …`         | the next line, all kinds               |
//!
//! When no `Kind` follows the directive, *all* issues on the target line are
//! suppressed. Kinds may be given by name (`UndefinedClass`) or by code
//! (`MIR0123`); multiple kinds are space- or comma-separated. PHPStan's
//! `@phpstan-ignore*` forms always suppress every kind on their target, since
//! PHPStan identifiers do not map onto mir's [`IssueKind`] names.
//!
//! [`IssueKind`]: mir_issues::IssueKind

use rustc_hash::{FxHashMap, FxHashSet};

/// Set of issue kinds a directive applies to.
#[derive(Debug, Clone)]
enum KindSet {
    /// Every kind on the target.
    All,
    /// Specific kinds, matched against `IssueKind::name()` or `code()`.
    Named(FxHashSet<String>),
}

impl KindSet {
    fn matches(&self, name: &str, code: &str) -> bool {
        match self {
            KindSet::All => true,
            KindSet::Named(set) => set.contains(name) || set.contains(code),
        }
    }

    fn merge(&mut self, other: KindSet) {
        match (self, other) {
            // Already broadest possible.
            (KindSet::All, _) => {}
            (slot @ KindSet::Named(_), KindSet::All) => *slot = KindSet::All,
            (KindSet::Named(a), KindSet::Named(b)) => a.extend(b),
        }
    }
}

/// Where a directive applies, relative to the comment's own line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    /// The comment's own physical line.
    SameLine,
    /// The next code line (next non-blank physical line).
    NextLine,
    /// Every line in the file.
    File,
}

struct Directive {
    scope: Scope,
    kinds: KindSet,
    /// For [`Scope::NextLine`]: whether to skip intervening comment lines (not
    /// just blanks) when locating the target. Set for "documents the following
    /// element" forms (`@psalm-suppress`, bare `@mir-ignore`, …) so a directive
    /// inside a multi-line docblock still lands on the declaration it annotates,
    /// past the closing `*/`.
    skip_comments: bool,
}

/// Per-file map of suppressed lines, built from source comments.
#[derive(Debug, Default)]
pub struct SuppressionMap {
    /// 1-based line number → kinds suppressed on that line.
    lines: FxHashMap<u32, KindSet>,
    /// Whole-file suppression, if any directive requested it.
    file: Option<KindSet>,
    /// Named (non-All) suppressions with their target lines, for
    /// `UnusedPsalmSuppress` detection. Each entry is `(target_line, kind_name)`.
    /// Only `@psalm-suppress X` / `@suppress X` / `@mir-suppress X` forms populate
    /// this — blanket `@phpstan-ignore*` suppressions are intentionally excluded.
    pub named_suppressions: Vec<(u32, String)>,
}

impl SuppressionMap {
    /// No directives — used to skip work for files with no suppression comments.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.file.is_none()
    }

    /// Whether an issue of `name`/`code` reported at 1-based `line` is suppressed.
    pub fn is_suppressed(&self, line: u32, name: &str, code: &str) -> bool {
        if let Some(file) = &self.file {
            if file.matches(name, code) {
                return true;
            }
        }
        self.lines.get(&line).is_some_and(|k| k.matches(name, code))
    }

    /// Scan `source` for suppression directives.
    pub fn from_source(source: &str) -> Self {
        let raw_lines: Vec<&str> = source.lines().collect();
        let mut map = SuppressionMap::default();

        for (idx, raw) in raw_lines.iter().enumerate() {
            let Some((directive, track_named)) = parse_directive_with_tracking(raw) else {
                continue;
            };
            match directive.scope {
                Scope::File => match &mut map.file {
                    Some(existing) => existing.merge(directive.kinds),
                    None => map.file = Some(directive.kinds),
                },
                Scope::SameLine => {
                    let line_no = idx as u32 + 1;
                    if track_named {
                        if let KindSet::Named(ref names) = directive.kinds {
                            for name in names {
                                map.named_suppressions.push((line_no, name.clone()));
                            }
                        }
                    }
                    insert_line(&mut map.lines, line_no, directive.kinds);
                }
                Scope::NextLine => {
                    let target = next_code_line(&raw_lines, idx, directive.skip_comments);
                    if track_named {
                        if let KindSet::Named(ref names) = directive.kinds {
                            for name in names {
                                map.named_suppressions.push((target, name.clone()));
                            }
                        }
                    }
                    insert_line(&mut map.lines, target, directive.kinds);
                }
            }
        }

        map
    }

    /// Returns unused named suppressions: those that did not match any issue
    /// in `all_issues`. The returned vec contains `(target_line, kind_name)`.
    ///
    /// `pre_suppressed` is the subset of `all_issues` that arrived already
    /// suppressed (via the `IssueBuffer` mechanism in collector/body analysis).
    /// These may be emitted at a different line than the suppression target
    /// (e.g. `InvalidDocblock` at a docblock-start line vs. the following
    /// declaration line), so they are matched within a 30-line window before
    /// the target.
    pub fn unused_named(
        &self,
        all_issues: &[mir_issues::Issue],
        pre_suppressed: &[&mir_issues::Issue],
    ) -> Vec<(u32, String)> {
        self.named_suppressions
            .iter()
            .filter(|(target_line, kind)| {
                let kind_matches = |issue: &&mir_issues::Issue| {
                    issue.kind.name() == kind.as_str() || issue.kind.code() == kind.as_str()
                };
                // Normal case: SuppressionMap-suppressed issue at the exact target line.
                let at_target = all_issues
                    .iter()
                    .any(|issue| issue.location.line == *target_line && kind_matches(&issue));
                if at_target {
                    return false; // suppression IS used
                }
                // Docblock case: collector-emitted issues (like `InvalidDocblock`)
                // land at the docblock-start line, which precedes the declaration
                // that the suppression targets. Allow a 30-line look-back so a
                // `@psalm-suppress InvalidDocblock` in a multi-line docblock is
                // recognised as used even though its issue line != target_line.
                let min_line = target_line.saturating_sub(30);
                let covered_by_pre_suppressed = pre_suppressed.iter().any(|issue| {
                    issue.location.line >= min_line
                        && issue.location.line < *target_line
                        && kind_matches(issue)
                });
                !covered_by_pre_suppressed
            })
            .cloned()
            .collect()
    }

    /// Like `unused_named` but takes a slice of `Issue` references.
    pub fn unused_named_ref(&self, issues: &[&mir_issues::Issue]) -> Vec<(u32, String)> {
        self.named_suppressions
            .iter()
            .filter(|(line, kind)| {
                !issues.iter().any(|issue| {
                    issue.location.line == *line
                        && (issue.kind.name() == kind || issue.kind.code() == kind)
                })
            })
            .cloned()
            .collect()
    }
}

fn insert_line(lines: &mut FxHashMap<u32, KindSet>, line: u32, kinds: KindSet) {
    match lines.get_mut(&line) {
        Some(existing) => existing.merge(kinds),
        None => {
            lines.insert(line, kinds);
        }
    }
}

/// Locate a directive's target line strictly after `idx`, as a 1-based number.
///
/// Always skips blank lines. When `skip_comments` is set, also skips
/// comment-only lines (`//`, `#`, `/* … */`, ` * …` docblock bodies and the
/// closing `*/`) so a directive written inside a multi-line docblock lands on
/// the declaration that follows it. Falls back to `idx + 2` when nothing
/// qualifies, so the directive still has a deterministic target.
fn next_code_line(raw_lines: &[&str], idx: usize, skip_comments: bool) -> u32 {
    for (offset, line) in raw_lines.iter().enumerate().skip(idx + 1) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if skip_comments && is_comment_only(trimmed) {
            continue;
        }
        return offset as u32 + 1;
    }
    idx as u32 + 2
}

/// Whether a trimmed line is purely a comment (no PHP code). `#[` is treated as
/// a PHP 8 attribute (code), not a `#` comment.
fn is_comment_only(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with("/*")
        || trimmed.starts_with('*')
        || (trimmed.starts_with('#') && !trimmed.starts_with("#["))
}

/// Directive keyword table, ordered longest-first so that, e.g.,
/// `@mir-ignore-next-line` is matched before the `@mir-ignore` prefix.
///
/// Each entry is `(keyword, scope, force_all)`. `force_all` makes the directive
/// suppress every kind regardless of trailing tokens (PHPStan semantics).
const KEYWORDS: &[(&str, Scope, bool)] = &[
    ("@mir-ignore-next-line", Scope::NextLine, false),
    ("@mir-suppress-next-line", Scope::NextLine, false),
    ("@phpstan-ignore-next-line", Scope::NextLine, true),
    ("@mir-ignore-line", Scope::SameLine, false),
    ("@mir-suppress-line", Scope::SameLine, false),
    ("@phpstan-ignore-line", Scope::SameLine, true),
    ("@mir-ignore-file", Scope::File, false),
    ("@mir-suppress-file", Scope::File, false),
    // Bare forms (scope resolved below from comment position).
    ("@mir-ignore", Scope::NextLine, false),
    ("@mir-suppress", Scope::NextLine, false),
    ("@psalm-suppress", Scope::NextLine, false),
    ("@suppress", Scope::NextLine, false),
    ("@phpstan-ignore", Scope::NextLine, true),
];

/// Bare directives (no `-line`/`-next-line`/`-file` suffix) resolve their scope
/// from where the comment sits: a trailing comment annotates its own line, a
/// standalone comment annotates the statement that follows it.
const BARE_KEYWORDS: &[&str] = &[
    "@mir-ignore",
    "@mir-suppress",
    "@psalm-suppress",
    "@suppress",
    "@phpstan-ignore",
];

/// Like `parse_directive` (which is parse_directive_with_tracking discarding the tracking flag),
/// but also returns whether named suppression tracking
/// should be applied (true for `@psalm-suppress`, `@mir-suppress`, `@suppress`
/// and `@mir-ignore` forms; false for `@phpstan-*` which are blanket suppressors
/// not tied to specific issue kinds).
fn parse_directive_with_tracking(raw: &str) -> Option<(Directive, bool)> {
    let comment = extract_comment(raw)?;

    for &(keyword, scope, force_all) in KEYWORDS {
        let Some(pos) = comment.content.find(keyword) else {
            continue;
        };
        // Reject keyword matches that are really a prefix of a longer token
        // (e.g. `@mir-ignore` inside `@mir-ignore-line`).
        let after = &comment.content[pos + keyword.len()..];
        if after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            continue;
        }

        let is_bare = BARE_KEYWORDS.contains(&keyword);

        // Bare forms: a trailing comment suppresses its own line.
        let scope = if is_bare && comment.has_code_before {
            Scope::SameLine
        } else {
            scope
        };

        // The "documents the following element" forms (bare `@psalm-suppress`,
        // `@mir-ignore`, …) skip past intervening comment lines — e.g. the
        // closing `*/` of a multi-line docblock — to reach the declaration.
        // PHPStan's explicit `*-next-line` and bare `@phpstan-ignore` keep their
        // literal next-non-blank-line semantics.
        let skip_comments = scope == Scope::NextLine && is_bare && !force_all;

        let kinds = if force_all {
            KindSet::All
        } else {
            parse_kinds(after)
        };

        // Track named suppressions only for non-phpstan forms (phpstan forms
        // always suppress all kinds, so they can never be "unused for a specific kind").
        let track_named = !keyword.starts_with("@phpstan");

        return Some((
            Directive {
                scope,
                kinds,
                skip_comments,
            },
            track_named,
        ));
    }

    None
}

struct Comment<'a> {
    /// Text from the comment introducer onward (still includes `*/`, `*`, etc.).
    content: &'a str,
    /// Whether non-whitespace code precedes the comment on the same line.
    has_code_before: bool,
}

/// Isolate the comment portion of a physical line, if any. Handles `//`, `#`
/// and `/* … */` introducers, block-comment continuation lines (` * …`) and
/// bare directive lines inside block comments (`@psalm-suppress …`).
fn extract_comment(raw: &str) -> Option<Comment<'_>> {
    let trimmed = raw.trim_start();

    // Block-comment continuation or a bare directive line: no code precedes it.
    if trimmed.starts_with('*') {
        return Some(Comment {
            content: trimmed.trim_start_matches('*'),
            has_code_before: false,
        });
    }
    if trimmed.starts_with('@') {
        return Some(Comment {
            content: trimmed,
            has_code_before: false,
        });
    }

    // Earliest single-line / block introducer on the line.
    let pos = [raw.find("//"), raw.find('#'), raw.find("/*")]
        .into_iter()
        .flatten()
        .min()?;
    let has_code_before = !raw[..pos].trim().is_empty();
    Some(Comment {
        content: &raw[pos..],
        has_code_before,
    })
}

/// Collect issue kind names/codes following a directive keyword. Stops at the
/// block-comment terminator and ignores non-identifier tokens. An empty result
/// means "all kinds".
fn parse_kinds(rest: &str) -> KindSet {
    let mut set = FxHashSet::default();
    for token in rest.split([' ', '\t', ',']) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        // End of the comment / docblock — stop scanning.
        if token.starts_with("*/") || token.starts_with('*') {
            break;
        }
        // A kind name is alphanumeric (plus `_`); anything else (a PHPStan
        // identifier like `argument.type`, prose, etc.) is skipped.
        if token.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            set.insert(token.to_string());
        }
    }
    if set.is_empty() {
        KindSet::All
    } else {
        KindSet::Named(set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(src: &str) -> SuppressionMap {
        SuppressionMap::from_source(src)
    }

    #[test]
    fn line_comment_above_statement_suppresses_next_line() {
        // line 2 comment → suppress line 3
        let m = map("<?php\n// @psalm-suppress UndefinedClass\nnew NoSuchClass();\n");
        assert!(m.is_suppressed(3, "UndefinedClass", "MIR0000"));
        assert!(!m.is_suppressed(2, "UndefinedClass", "MIR0000"));
    }

    #[test]
    fn trailing_comment_suppresses_own_line() {
        let m = map("<?php\nnew NoSuchClass(); // @mir-ignore UndefinedClass\n");
        assert!(m.is_suppressed(2, "UndefinedClass", "MIR0000"));
    }

    #[test]
    fn single_line_docblock_above_statement() {
        let m = map("<?php\n/** @psalm-suppress UndefinedClass */\nnew NoSuchClass();\n");
        assert!(m.is_suppressed(3, "UndefinedClass", "MIR0000"));
    }

    #[test]
    fn phpstan_ignore_next_line_suppresses_all() {
        let m = map("<?php\n// @phpstan-ignore-next-line\nnew NoSuchClass();\n");
        assert!(m.is_suppressed(3, "UndefinedClass", "MIR0000"));
        assert!(m.is_suppressed(3, "AnyOtherKind", "MIR9999"));
    }

    #[test]
    fn ignore_line_targets_own_line() {
        let m = map("<?php\nnew NoSuchClass(); // @mir-ignore-line\n");
        assert!(m.is_suppressed(2, "UndefinedClass", "MIR0000"));
    }

    #[test]
    fn next_line_skips_blank_lines() {
        let m = map("<?php\n/** @psalm-suppress UndefinedClass */\n\n\nnew NoSuchClass();\n");
        assert!(m.is_suppressed(5, "UndefinedClass", "MIR0000"));
    }

    #[test]
    fn multiline_docblock_skips_to_declaration() {
        // line 2: /**, line 3: * @psalm-suppress, line 4: */, line 5: declaration.
        let src =
            "<?php\n/**\n * @psalm-suppress UnusedMethod\n */\nprivate function a(): void {}\n";
        let m = map(src);
        assert!(m.is_suppressed(5, "UnusedMethod", "MIR0000"));
    }

    #[test]
    fn phpstan_next_line_is_literal_not_comment_skipping() {
        // PHPStan's -next-line targets the next non-blank line even if it's a
        // comment; it does not hunt for the next code line.
        let m = map("<?php\n// @phpstan-ignore-next-line\n// unrelated comment\nfoo();\n");
        assert!(m.is_suppressed(3, "X", "MIR0000"));
        assert!(!m.is_suppressed(4, "X", "MIR0000"));
    }

    #[test]
    fn named_kind_does_not_suppress_other_kinds() {
        let m = map("<?php\n// @mir-ignore UndefinedClass\nfoo();\n");
        assert!(m.is_suppressed(3, "UndefinedClass", "MIR0000"));
        assert!(!m.is_suppressed(3, "UndefinedFunction", "MIR0001"));
    }

    #[test]
    fn match_by_code() {
        let m = map("<?php\n// @mir-ignore MIR1400\nfoo();\n");
        assert!(m.is_suppressed(3, "ParseError", "MIR1400"));
    }

    #[test]
    fn file_scope_suppresses_every_line() {
        let m = map("<?php // @mir-ignore-file UndefinedClass\nfoo();\nbar();\n");
        assert!(m.is_suppressed(2, "UndefinedClass", "MIR0000"));
        assert!(m.is_suppressed(99, "UndefinedClass", "MIR0000"));
        assert!(!m.is_suppressed(2, "UndefinedFunction", "MIR0001"));
    }

    #[test]
    fn multiple_kinds_one_directive() {
        let m = map("<?php\n// @psalm-suppress UndefinedClass, NullMethodCall\nfoo();\n");
        assert!(m.is_suppressed(3, "UndefinedClass", "MIR0000"));
        assert!(m.is_suppressed(3, "NullMethodCall", "MIR0001"));
    }

    #[test]
    fn no_directive_is_empty() {
        let m = map("<?php\n$x = \"@psalm-suppress not a comment\";\nfoo();\n");
        // It's inside a string but after `//`? No `//` here, so not detected.
        assert!(m.is_empty());
    }

    #[test]
    fn prefix_is_not_confused_with_longer_keyword() {
        // `@mir-ignore-next-line` must be parsed as next-line, not bare same-line.
        let m = map("<?php\nfoo(); // @mir-ignore-next-line\nbar();\n");
        assert!(m.is_suppressed(3, "AnyKind", "MIR0000"));
        assert!(!m.is_suppressed(2, "AnyKind", "MIR0000"));
    }
}
