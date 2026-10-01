//! Test utilities for fixture-based testing.
//!
//! # Fixture formats
//!
//! **Single-file** (`===file===`, appears exactly once):
//! ```text
//! ===file===
//! <?php
//! ...
//! ===expect===
//! UndefinedMethod: Method Foo::bar() does not exist
//! ```
//!
//! **Multi-file** (`===file:name===`, one or more):
//! ```text
//! ===file:Base.php===
//! <?php
//! class Base { ... }
//! ===file:Child.php===
//! <?php
//! class Child extends Base { ... }
//! ===expect===
//! Child.php: UndefinedMethod: Method Child::bar() does not exist
//! ```
//!
//! **With config** (optional `===config===` section, must appear before file sections):
//! ```text
//! ===config===
//! php_version=8.1
//! suppress=MissingThrowsDocblock,UnusedFunction
//! stub_file=stubs/helpers.php
//! stub_dir=stubs
//! ===file===
//! <?php
//! ...
//! ===expect===
//! ...
//! ```
//!
//! `stub_file=path` and `stub_dir=path` refer to files/directories already declared
//! with `===file:path===` markers. They are wired into the session via
//! `AnalysisSession::with_user_stubs` and excluded from the analysis file list, so only the non-stub PHP
//! files are analysed. Multiple `stub_file=` and `stub_dir=` lines are allowed.
//!
//! **With Composer/PSR-4**:
//! ```text
//! ===file:composer.json===
//! {"autoload":{"psr-4":{"App\\":"src/"}}}
//! ===file:src/Base.php===
//! <?php
//! namespace App;
//! class Base { ... }
//! ===file:Child.php===
//! <?php
//! class Child extends \App\Base { ... }
//! ===expect===
//! Child.php: UndefinedMethod: Method Child::bar() does not exist
//! ```
//!
//! **With description** (optional `===description===` section, must appear before file sections):
//! ```text
//! ===description===
//! Verify that calling a method on a null variable is reported.
//! ===file===
//! <?php
//! ...
//! ===expect===
//! ...
//! ```
//!
//! **Skipped / WIP fixture** (`===ignore===`, must appear before file sections):
//! ```text
//! ===ignore===
//! ===file===
//! <?php
//! ...
//! ===expect===
//! ...
//! ```
//!
//! The presence of `===ignore===` causes the generated test to be marked
//! `#[ignore]` at compile time (via `build.rs`), so it shows up as `ignored`
//! rather than `ok` or `FAILED` in test output.
//!
//! **Editor query** (`===cursor===`, must appear before file sections):
//! ```text
//! ===cursor===
//! references include_declaration
//! ===file===
//! <?php
//! function greet(): void {}
//! gr<CURSOR>eet();
//! ===expect===
//! test.php@2:9-2:14
//! test.php@3:0-3:5
//! ```
//!
//! The files are analyzed as a project, then the query runs at the `<CURSOR>`
//! marker, which is removed from the source first and must appear exactly once
//! across all files. Queries exercise the resolution primitives editor
//! front-ends build on; presentation (hover text, docblock rendering) is theirs.
//!
//! - `symbol` — the `ResolvedSymbol` at the cursor from `FileAnalysis::symbol_at`
//!   as `kind: K` and `type: T`; must agree with `AnalysisSession::symbol_at`.
//! - `definition` — `name_at`, then the declaration's `LOCATION` from
//!   `definition_of_cached`.
//! - `references [include_declaration] [use_imports]` — `name_at`, then one
//!   `LOCATION` per `indexed_references_to` hit, sorted, searched across every
//!   non-stub PHP file.
//!
//! A `LOCATION` is `path@line:col-line_end:col_end`, with `path` relative to
//! the fixture root (stub declarations keep their `stubs/…` path). A failed
//! lookup renders as `error: NotFound` or `error: NoSourceLocation`.
//!
//! **Annotations** (diagnostic fixtures): a `// ^^^ Kind: message` comment line
//! under a source line expects that issue on the line above, starting at the
//! first caret's column and spanning the carets. `//<^^^` starts at column 0.
//! A `+N:C` token after the carets (`// ^ +2:1 Kind`) ends the span at column
//! `C` of the source line `N` lines below (annotation lines don't count). A column 1 start needs `===expect===`.
//! Annotations combine with `===expect===`; `UPDATE_FIXTURES=1` leaves matched
//! ones out of the rewritten section.
//! ```text
//! ===file===
//! <?php
//! function f(Missing $x): void {}
//! //         ^^^^^^^ UndefinedClass: Class Missing does not exist
//! ===expect===
//! ```
//!
//! **Edits** (`===edit:name===`, after the file sections, multi-file only):
//! ```text
//! ===file:Lib.php===
//! <?php
//! function lib(): int { return 1; }
//! ===file:Use.php===
//! <?php
//! function run(): int { return lib(); }
//! ===expect===
//! <<none>>
//! ===edit:Lib.php===
//! <?php
//! function lib(): string { return ''; }
//! ===expect===
//! Use.php: InvalidReturnType@2:22-2:35: Return type 'string' is not compatible with declared 'int'
//! ```
//!
//! Replays an editor session: the files are ingested and warmed on a
//! snapshot and their diagnostics are asserted against the first
//! `===expect===`; then each edit replaces its file's text in place and the
//! diagnostics are asserted against the second, edited files first. Both
//! cover open-file and class-level issues.
//!
//! # Validation rules
//!
//! - `===file===` (bare, no name) must appear **at most once** per fixture.
//! - `===file===` and `===file:name===` cannot appear in the same fixture.
//! - A fixture with no file section at all fails immediately.
//! - `===config===` must appear **at most once** per fixture.
//! - Every key in `===config===` must be a recognised key (`php_version`,
//!   `suppress`, `stub_file`, `stub_dir`, `memoize_method_call_results`); unknown keys
//!   fail the test.
//! - `php_version` is parsed via [`std::str::FromStr`] on [`PhpVersion`] (same parser as the
//!   real CLI config); invalid values fail the test.
//! - `suppress` accepts a comma-separated list of [`IssueKind`] names to drop
//!   from the analyzer's output. Every other kind is asserted strictly: a
//!   fixture must either expect each issue its example code produces or list
//!   the kind here. The dead-code group (`UnusedFunction`/`UnusedMethod`/
//!   `UnusedProperty`) is the one exception — it is suppressed by default
//!   (merged on top of any explicit `suppress=`) so a fixture's bare top-level
//!   functions don't emit unsolicited noise. That default is held back only
//!   when the fixture's `===expect===` references one of those kinds, which is
//!   how a fixture opts in to dead-code reporting.
//! - `stub_file` and `stub_dir` accept a relative path (matching a `===file:===` name).
//! - `===description===` must appear **at most once** and before any file section.
//! - `===ignore===` must appear **at most once** and before any file section.
//! - `===cursor===` must appear **at most once** and before any file section,
//!   and only together with a `<CURSOR>` marker; `suppress` is rejected there.
//! - `===edit:name===` must follow every file section and name a declared,
//!   analyzed project file; it can't be combined with `===cursor===`.
//! - `===expect===` appears exactly once, except in edit fixtures: there it
//!   appears twice, between the file and edit sections and after the edits.
//!   Each edit-fixture section lists issues or holds `<<none>>` alone.
//!
//! # Expect format
//!
//! Single-file fixtures use `KindName@line:col: message`.
//! Multi-file fixtures use `FileName.php: KindName@line:col: message`.
//! `===cursor===` fixtures use the query output described above.
//!
//! Location assertions (`@line:col`) are **required**. Both line and column must be specified
//! and must match for the issue to be considered a match.
//!
//! Set `UPDATE_FIXTURES=1` to rewrite the expect section with actual output (including locations).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::{batch::BatchOptions, session::AnalysisSession, PhpVersion, ReferenceIncludes};
use mir_issues::{Issue, IssueKind};
use parking_lot::Mutex;

static COUNTER: AtomicU64 = AtomicU64::new(0);

// ---------------------------------------------------------------------------
// Fixture configuration
// ---------------------------------------------------------------------------

#[derive(Default)]
struct FixtureConfig {
    php_version: Option<PhpVersion>,
    /// Issue kinds to drop from the analyzer's output, set from the
    /// `suppress=Foo,Bar` config key. The runner additionally merges in the
    /// dead-code group by default (see `run_fixture`) unless the fixture
    /// expects a dead-code diagnostic.
    suppressed_issue_kinds: Option<rustc_hash::FxHashSet<String>>,
    /// Paths (relative to temp dir) to pass as `analyzer.stub_files`.
    stub_files: Vec<String>,
    /// Paths (relative to temp dir) to pass as `analyzer.stub_dirs`.
    stub_dirs: Vec<String>,
    memoize_method_call_results: bool,
}

// ---------------------------------------------------------------------------
// Public inline-analysis API
// ---------------------------------------------------------------------------

/// Run the full analyzer on an inline PHP string and return all unsuppressed issues.
pub fn check(src: &str) -> Vec<Issue> {
    run_analyzer(&[("test.php", src)], &FixtureConfig::default())
}

/// Analyze a set of named PHP files together, returning all unsuppressed issues.
///
/// Each entry is `(filename, php_source)`. Files are written to a unique temp
/// directory, analyzed together, then cleaned up.
///
/// If a `"composer.json"` entry is included, a `Psr4Map` is built from it.
/// Files under PSR-4-mapped directories are left for lazy discovery and are
/// **not** passed to `analyze()` explicitly.
pub fn check_files(files: &[(&str, &str)]) -> Vec<Issue> {
    run_analyzer(files, &FixtureConfig::default())
}

// ---------------------------------------------------------------------------
// Fixture data types
// ---------------------------------------------------------------------------

/// One expected issue from a `.phpt` fixture's `===expect===` section.
pub(crate) struct ExpectedIssue {
    pub file: Option<String>,
    pub kind_name: String,
    pub message: String,
    pub line: Option<u32>,
    pub col_start: Option<u16>,
    pub line_end: Option<u32>,
    pub col_end: Option<u16>,
}

/// Parsed representation of a `.phpt` fixture.
pub(crate) struct ParsedFixture {
    /// `(filename, content)` pairs — always at least one entry.
    pub files: Vec<(String, String)>,
    pub expected: Vec<ExpectedIssue>,
    pub is_multi: bool,
    /// Optional human-readable description from `===description===`.
    pub description: Option<String>,
    /// Set for `===cursor===` fixtures, whose `expected` issue list stays empty.
    cursor: Option<Cursor>,
    /// `(filename, new content)` from `===edit:name===`, applied in order.
    edits: Vec<(String, String)>,
    /// Edit fixtures' `===expect===` issues before the first edit.
    expected_before_edits: Vec<ExpectedIssue>,
    config: FixtureConfig,
}

/// Editor query a `===cursor===` fixture runs at its `<CURSOR>` marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorQuery {
    Symbol,
    Definition,
    References {
        include_declaration: bool,
        includes: ReferenceIncludes,
    },
}

struct Cursor {
    query: CursorQuery,
    /// Index into [`ParsedFixture::files`] of the file holding the marker.
    file: usize,
    /// Byte offset of the marker in that file, after the marker is removed.
    offset: u32,
    expected: Vec<String>,
}

// ---------------------------------------------------------------------------
// Fixture parsing
// ---------------------------------------------------------------------------

const BARE_FILE: &str = "===file===";
const FILE_PREFIX: &str = "===file:";
const EDIT_PREFIX: &str = "===edit:";
const CONFIG_MARKER: &str = "===config===";
const EXPECT_MARKER: &str = "===expect===";
const DESCRIPTION_MARKER: &str = "===description===";
const IGNORE_MARKER: &str = "===ignore===";
const CURSOR_MARKER: &str = "===cursor===";
const CURSOR: &str = "<CURSOR>";
/// Body of an edit fixture's `===expect===` section that expects no issues.
const NO_ISSUES: &str = "<<none>>";

/// Parse a `.phpt` fixture file.
pub(crate) fn parse_phpt(content: &str, path: &str) -> ParsedFixture {
    // --- Locate the final expect (required; edit fixtures also have one before their edits) ---
    let expect_count = count_occurrences(content, EXPECT_MARKER);
    let edit_fixture = content.contains(EDIT_PREFIX);
    let wanted = if edit_fixture { 2 } else { 1 };
    assert_eq!(
        expect_count,
        wanted,
        "fixture {path}: {EXPECT_MARKER} must appear {} (found {expect_count})",
        if edit_fixture {
            "twice: before the first ===edit:name=== and after the edits"
        } else {
            "exactly once"
        }
    );
    let expect_pos = content.rfind(EXPECT_MARKER).unwrap();
    let header_region = &content[..expect_pos];
    let expect_content = content[expect_pos + EXPECT_MARKER.len()..].trim();

    // --- Validate header sections ---
    // They must appear before any file marker so their text is never silently
    // included in the PHP source of the first file.
    for marker in [
        CONFIG_MARKER,
        DESCRIPTION_MARKER,
        IGNORE_MARKER,
        CURSOR_MARKER,
    ] {
        let count = count_occurrences(header_region, marker);
        assert!(
            count <= 1,
            "fixture {path}: {marker} must appear at most once, found {count} times"
        );
        if let (Some(pos), Some(first_file_pos)) =
            (header_region.find(marker), header_region.find("===file"))
        {
            assert!(
                pos < first_file_pos,
                "fixture {path}: {marker} must appear before the first ===file=== / ===file:name=== marker"
            );
        }
    }

    // --- Count and validate file markers ---
    let bare_count = count_occurrences(header_region, BARE_FILE);
    // FILE_PREFIX ("===file:") won't match BARE_FILE ("===file===") since after
    // "file" one has ':' and the other '='.
    let named_count = count_occurrences(header_region, FILE_PREFIX);

    assert!(
        !(bare_count > 0 && named_count > 0),
        "fixture {path}: cannot mix ===file=== and ===file:name=== markers in the same fixture"
    );
    assert!(
        bare_count > 0 || named_count > 0,
        "fixture {path}: no ===file=== or ===file:name=== section found"
    );
    assert!(
        bare_count <= 1,
        "fixture {path}: ===file=== must appear at most once, found {bare_count} times"
    );

    let is_multi = named_count > 0;

    let edits = extract_named_sections(header_region, EDIT_PREFIX, path);
    if let Some(first_edit) = header_region.find(EDIT_PREFIX) {
        assert!(
            is_multi,
            "fixture {path}: {EDIT_PREFIX}name=== needs {FILE_PREFIX}name=== sections"
        );
        assert!(
            header_region.rfind(FILE_PREFIX).unwrap() < first_edit,
            "fixture {path}: {EDIT_PREFIX}name=== must follow every file section"
        );
        assert!(
            section_body(header_region, CURSOR_MARKER).is_none(),
            "fixture {path}: {EDIT_PREFIX}name=== can't be combined with {CURSOR_MARKER}"
        );
    }
    let expected_before_edits = match header_region.find(EXPECT_MARKER) {
        Some(pos) => {
            let first_edit = header_region.find(EDIT_PREFIX);
            assert!(
                first_edit.is_some_and(|edit| header_region.rfind(FILE_PREFIX).unwrap() < pos
                    && pos < edit),
                "fixture {path}: the first {EXPECT_MARKER} must sit between the file and edit sections"
            );
            let body = &header_region[pos + EXPECT_MARKER.len()..first_edit.unwrap()];
            parse_edit_expect_lines(body, is_multi, path)
        }
        None => Vec::new(),
    };

    // --- Extract file content(s) ---
    let mut files = if is_multi {
        extract_named_sections(header_region, FILE_PREFIX, path)
    } else {
        let bare_pos = header_region.find(BARE_FILE).unwrap();
        let src = header_region[bare_pos + BARE_FILE.len()..]
            .trim()
            .to_string();
        vec![("test.php".to_string(), src)]
    };

    let config = section_body(header_region, CONFIG_MARKER)
        .map(|text| parse_config_section(text, path))
        .unwrap_or_default();
    let description = section_body(header_region, DESCRIPTION_MARKER).map(str::to_string);

    let expect_lines = meaningful_lines(expect_content);

    for (name, _) in &edits {
        assert!(
            files.iter().any(|(file, _)| file == name),
            "fixture {path}: {EDIT_PREFIX}{name}=== names no {FILE_PREFIX}{name}=== section"
        );
    }

    let marker = take_cursor_marker(&mut files, path);
    let (expected, cursor) = match (section_body(header_region, CURSOR_MARKER), marker) {
        (Some(query), Some((file, offset))) => {
            assert!(
                config.suppressed_issue_kinds.is_none(),
                "fixture {path}: suppress has no effect in a ===cursor=== fixture"
            );
            let cursor = Cursor {
                query: parse_cursor_query(query, path),
                file,
                offset,
                expected: expect_lines.map(str::to_string).collect(),
            };
            (Vec::new(), Some(cursor))
        }
        (Some(_), None) => panic!("fixture {path}: ===cursor=== needs a {CURSOR} marker"),
        (None, Some(_)) => panic!("fixture {path}: {CURSOR} marker needs a ===cursor=== section"),
        (None, None) if edit_fixture => (
            parse_edit_expect_lines(expect_content, is_multi, path),
            None,
        ),
        (None, None) => {
            let mut expected = parse_expect_lines(expect_content, is_multi, path);
            expected.extend(parse_annotations(&files, is_multi, path));
            (expected, None)
        }
    };

    ParsedFixture {
        files,
        expected,
        is_multi,
        description,
        cursor,
        edits,
        expected_before_edits,
        config,
    }
}

/// Expected issues from `// ^^^ Kind: message` lines under a source line.
/// Carets give the 0-based start column and length on the nearest preceding
/// non-annotation line. `//<^^^` starts at column 0; a `+N:C` token after the
/// carets ends the span at column `C` of the source line `N` lines below
/// (annotation lines don't count).
fn parse_annotations(files: &[(String, String)], is_multi: bool, path: &str) -> Vec<ExpectedIssue> {
    let mut out = Vec::new();
    for (name, src) in files {
        let lines: Vec<&str> = src.lines().collect();
        let mut target: Option<u32> = None;
        for (idx, &text) in lines.iter().enumerate() {
            let Some((col, len, rest)) = split_annotation(text) else {
                target = Some(idx as u32 + 1);
                continue;
            };
            let line = target.unwrap_or_else(|| {
                panic!(
                    "fixture {path}: annotation on line {} has no source line above it",
                    idx + 1
                )
            });
            let (line_end, col_end, rest) = match parse_span_end(rest) {
                Some((lines_below, end_col, rest)) => (
                    nth_source_line_after(&lines, line, lines_below),
                    end_col,
                    rest,
                ),
                None => (line, col + len, rest),
            };
            let (kind, message) = match rest.split_once(": ") {
                Some((kind, message)) => (kind, message.trim()),
                None => (rest, ""),
            };
            assert!(
                !kind.is_empty(),
                "fixture {path}: annotation on line {} needs a Kind after the carets",
                idx + 1
            );
            out.push(ExpectedIssue {
                file: is_multi.then(|| name.clone()),
                kind_name: kind.to_string(),
                message: message.to_string(),
                line: Some(line),
                col_start: Some(col),
                line_end: Some(line_end),
                col_end: Some(col_end),
            });
        }
    }
    out
}

/// 1-based file line of the `n`th source (non-annotation) line after `line`.
fn nth_source_line_after(lines: &[&str], line: u32, n: u32) -> u32 {
    let mut remaining = n;
    let mut at = line;
    while remaining > 0 && (at as usize) < lines.len() {
        at += 1;
        if split_annotation(lines[at as usize - 1]).is_none() {
            remaining -= 1;
        }
    }
    at
}

/// `(lines below, end column, remaining text)` for a leading `+N:C` token.
fn parse_span_end(rest: &str) -> Option<(u32, u16, &str)> {
    let body = rest.strip_prefix('+')?;
    let (span, remaining) = body.split_once(' ').unwrap_or((body, ""));
    let (lines_below, end_col) = span.split_once(':')?;
    Some((
        lines_below.parse().ok()?,
        end_col.parse().ok()?,
        remaining.trim(),
    ))
}

/// `(caret column, caret count, text after the carets)` for an annotation line.
fn split_annotation(text: &str) -> Option<(u16, u16, &str)> {
    let after_slashes = text.trim_start().strip_prefix("//")?;
    let (carets_at, col) = match after_slashes.strip_prefix('<') {
        Some(rest) => (rest, 0),
        None => {
            let carets_at = after_slashes.trim_start();
            (
                carets_at,
                (text.chars().count() - carets_at.chars().count()) as u16,
            )
        }
    };
    if !carets_at.starts_with('^') {
        return None;
    }
    let len = carets_at.chars().take_while(|&c| c == '^').count();
    let rest = carets_at.trim_start_matches('^').trim();
    Some((col, len as u16, rest))
}

fn meaningful_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

fn parse_expect_lines(text: &str, is_multi: bool, path: &str) -> Vec<ExpectedIssue> {
    meaningful_lines(text)
        .map(|l| {
            if is_multi {
                parse_multi_expect_line(l, path)
            } else {
                parse_single_expect_line(l, path)
            }
        })
        .collect()
}

/// Trimmed text between `marker` and the next section marker.
fn section_body<'a>(region: &'a str, marker: &str) -> Option<&'a str> {
    let start = region.find(marker)? + marker.len();
    let end = region[start..]
        .find("\n===")
        .map_or(region.len(), |r| start + r);
    Some(region[start..end].trim())
}

/// Remove the `<CURSOR>` marker, returning its file index and byte offset.
fn take_cursor_marker(files: &mut [(String, String)], path: &str) -> Option<(usize, u32)> {
    let mut found = None;
    for (index, (_, src)) in files.iter_mut().enumerate() {
        let Some(offset) = src.find(CURSOR) else {
            continue;
        };
        assert!(
            found.is_none() && count_occurrences(src, CURSOR) == 1,
            "fixture {path}: {CURSOR} must appear exactly once across all files"
        );
        src.replace_range(offset..offset + CURSOR.len(), "");
        found = Some((index, offset as u32));
    }
    found
}

fn parse_cursor_query(text: &str, path: &str) -> CursorQuery {
    let mut words = text.split_whitespace();
    let query = words.next().unwrap_or_else(|| {
        panic!("fixture {path}: ===cursor=== needs a query — valid queries: symbol, definition, references")
    });
    let options: Vec<&str> = words.collect();
    match query {
        "symbol" | "definition" => {
            assert!(
                options.is_empty(),
                "fixture {path}: {query} takes no options, found {options:?}"
            );
            if query == "symbol" {
                CursorQuery::Symbol
            } else {
                CursorQuery::Definition
            }
        }
        "references" => {
            let mut include_declaration = false;
            let mut includes = ReferenceIncludes::Plain;
            for option in options {
                match option {
                    "include_declaration" => include_declaration = true,
                    "use_imports" => includes = ReferenceIncludes::PlainAndUseImports,
                    other => panic!(
                        "fixture {path}: unknown references option {other:?} — valid options: include_declaration, use_imports"
                    ),
                }
            }
            CursorQuery::References {
                include_declaration,
                includes,
            }
        }
        other => panic!(
            "fixture {path}: unknown ===cursor=== query {other:?} — valid queries: symbol, definition, references"
        ),
    }
}

fn parse_config_section(text: &str, path: &str) -> FixtureConfig {
    let mut config = FixtureConfig::default();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line.split_once('=').unwrap_or_else(|| {
            panic!("fixture {path}: invalid config line {line:?} — expected key=value")
        });
        match key.trim() {
            "php_version" => {
                let v = value.trim().parse::<PhpVersion>().unwrap_or_else(|e| {
                    panic!("fixture {path}: invalid php_version: {e}")
                });
                config.php_version = Some(v);
            }
            "suppress" => {
                let set = config.suppressed_issue_kinds.get_or_insert_with(Default::default);
                for name in value.split(',') {
                    let trimmed = name.trim();
                    if !trimmed.is_empty() {
                        set.insert(trimmed.to_string());
                    }
                }
            }
            "stub_file" => {
                config.stub_files.push(value.trim().to_string());
            }
            "stub_dir" => {
                config.stub_dirs.push(value.trim().to_string());
            }
            "memoize_method_call_results" => {
                config.memoize_method_call_results = value.trim() == "true";
            }
            other => panic!(
                "fixture {path}: unknown config key {other:?} — valid keys: php_version, suppress, stub_file, stub_dir, memoize_method_call_results"
            ),
        }
    }
    config
}

/// `(name, content)` of every `{prefix}name===` section; content runs to the
/// next file, edit, or expect section.
fn extract_named_sections(region: &str, prefix: &str, path: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    let mut search_from = 0;

    while let Some(marker_rel) = region[search_from..].find(prefix) {
        let marker_abs = search_from + marker_rel;
        let after_prefix = marker_abs + prefix.len();

        let close_rel = region[after_prefix..]
            .find("===")
            .unwrap_or_else(|| panic!("fixture {path}: unclosed {prefix} marker"));

        let file_name = region[after_prefix..after_prefix + close_rel].to_string();
        let content_start = after_prefix + close_rel + "===".len();

        let content_end = [FILE_PREFIX, EDIT_PREFIX, EXPECT_MARKER]
            .iter()
            .filter_map(|next| region[content_start..].find(next))
            .min()
            .map_or(region.len(), |r| content_start + r);

        let file_content = region[content_start..content_end].trim().to_string();
        files.push((file_name, file_content));
        search_from = content_end;
    }

    files
}

fn parse_single_expect_line(line: &str, path: &str) -> ExpectedIssue {
    let parts: Vec<&str> = line.splitn(2, ": ").collect();
    let (kind_part, message) = match parts.len() {
        2 => (parts[0], parts[1].trim().to_string()),
        1 => (parts[0], String::new()),
        _ => panic!("fixture {path}: invalid expect line {line:?}"),
    };

    let (kind_name, line_col) = if let Some(at_pos) = kind_part.find('@') {
        (
            kind_part[..at_pos].trim().to_string(),
            Some(&kind_part[at_pos + 1..]),
        )
    } else {
        (kind_part.trim().to_string(), None)
    };

    let (line_num, col_start, line_end, col_end) = if let Some(loc) = line_col {
        // Format: "line:col" or "line:col-line_end:col_end"
        let (start_part, end_part) = if let Some(dash) = loc.find('-') {
            (&loc[..dash], Some(&loc[dash + 1..]))
        } else {
            (loc, None)
        };
        let loc_parts: Vec<&str> = start_part.split(':').collect();
        if loc_parts.len() != 2 {
            panic!("fixture {path}: invalid location format in {line:?} — expected \"@line:col\"");
        }
        let l = loc_parts[0]
            .parse::<u32>()
            .unwrap_or_else(|_| panic!("fixture {path}: invalid line number in {line:?}"));
        let c = loc_parts[1]
            .parse::<u16>()
            .unwrap_or_else(|_| panic!("fixture {path}: invalid column number in {line:?}"));
        let (le, ce) = if let Some(end) = end_part {
            let end_parts: Vec<&str> = end.split(':').collect();
            if end_parts.len() != 2 {
                panic!("fixture {path}: invalid end-location format in {line:?} — expected \"line_end:col_end\"");
            }
            let le = end_parts[0]
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("fixture {path}: invalid end line number in {line:?}"));
            let ce = end_parts[1].parse::<u16>().unwrap_or_else(|_| {
                panic!("fixture {path}: invalid end column number in {line:?}")
            });
            (Some(le), Some(ce))
        } else {
            (None, None)
        };
        (Some(l), Some(c), le, ce)
    } else {
        (None, None, None, None)
    };

    ExpectedIssue {
        file: None,
        kind_name,
        message,
        line: line_num,
        col_start,
        line_end,
        col_end,
    }
}

fn parse_multi_expect_line(line: &str, path: &str) -> ExpectedIssue {
    let parts: Vec<&str> = line.splitn(3, ": ").collect();
    assert!(
        parts.len() >= 2,
        "fixture {path}: invalid multi-file expect line {line:?} — expected \"FileName.php: KindName[@line:col][ : message]\""
    );

    let kind_part = parts[1];
    let message = if parts.len() >= 3 {
        parts[2].trim().to_string()
    } else {
        String::new()
    };

    let (kind_name, line_col) = if let Some(at_pos) = kind_part.find('@') {
        (
            kind_part[..at_pos].trim().to_string(),
            Some(&kind_part[at_pos + 1..]),
        )
    } else {
        (kind_part.trim().to_string(), None)
    };

    let (line_num, col_start, line_end, col_end) = if let Some(loc) = line_col {
        // Format: "line:col" or "line:col-line_end:col_end"
        let (start_part, end_part) = if let Some(dash) = loc.find('-') {
            (&loc[..dash], Some(&loc[dash + 1..]))
        } else {
            (loc, None)
        };
        let loc_parts: Vec<&str> = start_part.split(':').collect();
        if loc_parts.len() != 2 {
            panic!("fixture {path}: invalid location format in {line:?} — expected \"@line:col\"");
        }
        let l = loc_parts[0]
            .parse::<u32>()
            .unwrap_or_else(|_| panic!("fixture {path}: invalid line number in {line:?}"));
        let c = loc_parts[1]
            .parse::<u16>()
            .unwrap_or_else(|_| panic!("fixture {path}: invalid column number in {line:?}"));
        let (le, ce) = if let Some(end) = end_part {
            let end_parts: Vec<&str> = end.split(':').collect();
            if end_parts.len() != 2 {
                panic!("fixture {path}: invalid end-location format in {line:?} — expected \"line_end:col_end\"");
            }
            let le = end_parts[0]
                .parse::<u32>()
                .unwrap_or_else(|_| panic!("fixture {path}: invalid end line number in {line:?}"));
            let ce = end_parts[1].parse::<u16>().unwrap_or_else(|_| {
                panic!("fixture {path}: invalid end column number in {line:?}")
            });
            (Some(le), Some(ce))
        } else {
            (None, None)
        };
        (Some(l), Some(c), le, ce)
    } else {
        (None, None, None, None)
    };

    ExpectedIssue {
        file: Some(parts[0].trim().to_string()),
        kind_name,
        message,
        line: line_num,
        col_start,
        line_end,
        col_end,
    }
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    let mut count = 0;
    let mut start = 0;
    while let Some(pos) = haystack[start..].find(needle) {
        count += 1;
        start += pos + needle.len();
    }
    count
}

// ---------------------------------------------------------------------------
// Fixture runner
// ---------------------------------------------------------------------------

/// Run a `.phpt` fixture file and assert its output matches the `===expect===` section.
///
/// Set `UPDATE_FIXTURES=1` to rewrite the expect section with actual output.
pub fn run_fixture(path: &str) {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"));

    let mut fixture = parse_phpt(&content, path);
    match fixture.cursor.take() {
        Some(cursor) => run_cursor_fixture(path, &content, &fixture, &cursor),
        None if !fixture.edits.is_empty() => run_edit_fixture(path, &content, fixture),
        None => run_diagnostic_fixture(path, &content, fixture),
    }
}

const UPDATE_HINT: &str =
    "UPDATE_FIXTURES=1 cargo test -p mir-analyzer --test fixtures <fixture name>";

fn update_requested() -> bool {
    std::env::var("UPDATE_FIXTURES").as_deref() == Ok("1")
}

fn run_diagnostic_fixture(path: &str, content: &str, mut fixture: ParsedFixture) {
    suppress_dead_code_by_default(&mut fixture);
    let actual = run_analyzer(&file_refs(&fixture), &fixture.config);

    if update_requested() {
        let annotated = parse_annotations(&fixture.files, fixture.is_multi, path);
        let unannotated: Vec<Issue> = actual
            .into_iter()
            .filter(|a| !annotated.iter().any(|e| issue_matches(a, e)))
            .collect();
        rewrite_expect_section(
            path,
            content,
            &fmt_expect_lines(&unannotated, fixture.is_multi),
        );
        return;
    }

    assert_fixture(path, &fixture, &actual);
}

fn run_edit_fixture(path: &str, content: &str, mut fixture: ParsedFixture) {
    suppress_dead_code_by_default(&mut fixture);
    let suppressed = fixture
        .config
        .suppressed_issue_kinds
        .clone()
        .unwrap_or_default();
    let (before, after) =
        with_fixture_session(&file_refs(&fixture), &fixture.config, |session, ws| {
            let files: Vec<Arc<str>> = ws
                .analyzed
                .iter()
                .map(|p| Arc::from(p.to_string_lossy().as_ref()))
                .collect();
            let read = |file: &str| -> Arc<str> {
                std::fs::read_to_string(file)
                    .unwrap_or_else(|e| panic!("failed to read {file}: {e}"))
                    .into()
            };
            for file in &files {
                session.ingest_file(file.clone(), read(file));
            }
            session.prepare_for_query(None);
            let warmed = session
                .snapshot()
                .warm_files(&files, &crate::IndexCancel::new())
                .expect("uncancelled warm pass");
            assert!(warmed, "fixture {path}: warm pass stopped early");

            let diagnostics = |session: &mut AnalysisSession, order: &[&Arc<str>]| {
                order
                    .iter()
                    .flat_map(|file| {
                        let mut issues = session.analyze_file_diagnostics(file, &read(file)).issues;
                        issues.extend(session.class_issues(std::slice::from_ref(*file)));
                        issues
                    })
                    .filter(|i| !i.suppressed && !suppressed.contains(i.kind.display_name()))
                    .collect::<Vec<_>>()
            };
            let before = diagnostics(session, &files.iter().collect::<Vec<_>>());

            let mut edited: HashSet<Arc<str>> = HashSet::new();
            for (name, text) in &fixture.edits {
                let file = ws.dir.join(name);
                assert!(
                    ws.analyzed.contains(&file),
                    "fixture {path}: {EDIT_PREFIX}{name}=== must edit an analyzed project file"
                );
                std::fs::write(&file, text)
                    .unwrap_or_else(|e| panic!("failed to write {name}: {e}"));
                let file: Arc<str> = Arc::from(file.to_string_lossy().as_ref());
                session.ingest_file(file.clone(), Arc::from(text.as_str()));
                edited.insert(file);
            }

            let (mut order, rest): (Vec<_>, Vec<_>) =
                files.iter().partition(|file| edited.contains(*file));
            order.extend(rest);
            (before, diagnostics(session, &order))
        });

    if update_requested() {
        rewrite_edit_expect_sections(
            path,
            content,
            &fmt_expect_lines(&before, fixture.is_multi),
            &fmt_expect_lines(&after, fixture.is_multi),
        );
        return;
    }

    let mut failures = issue_failures(&fixture.expected_before_edits, &before, fixture.is_multi)
        .into_iter()
        .map(|f| format!("{f}  (before edits)"))
        .collect::<Vec<_>>();
    failures.extend(
        issue_failures(&fixture.expected, &after, fixture.is_multi)
            .into_iter()
            .map(|f| format!("{f}  (after edits)")),
    );
    if !failures.is_empty() {
        panic!(
            "fixture {path} FAILED:{}\n{}\n\nTo fix, run: {UPDATE_HINT}\n\nActual issues before edits:\n{}\n\nActual issues after edits:\n{}",
            fmt_description(&fixture),
            failures.join("\n"),
            fmt_issues(&before, fixture.is_multi),
            fmt_issues(&after, fixture.is_multi)
        );
    }
}

fn suppress_dead_code_by_default(fixture: &mut ParsedFixture) {
    // Auto-suppression: the dead-code group (UnusedMethod/Property/Function) is
    // suppressed by default so authors don't have to sprinkle boilerplate
    // `suppress=` lines on every fixture whose example code happens to declare
    // an uncalled global function. This default is applied *additively* — it
    // merges with any explicit `suppress=Foo,Bar` rather than being skipped when
    // one is present — and is held back in two cases, so the `dead_code_enabled`
    // path filter in `run_analyzer` keeps its semantics:
    //   1. the fixture expects a dead-code diagnostic, or
    //   2. the fixture sets an explicit *empty* `suppress=`, which is the marker
    //      for "report everything, including dead code" (used by the negative
    //      dead-code fixtures that assert a *used* symbol is not flagged).
    //
    // Every other diagnostic — including the noisy kinds like UnusedParam,
    // MixedArgument, or MissingParamType — is asserted strictly: a fixture must
    // either expect each issue its example code produces or list the kind in an
    // explicit `suppress=` config line.
    let dead = crate::batch::dead_code_issue_kinds();
    let expects_dead_code = fixture
        .expected
        .iter()
        .chain(&fixture.expected_before_edits)
        .any(|e| dead.contains(&e.kind_name.as_str()));
    let opts_into_dead_code = matches!(
        &fixture.config.suppressed_issue_kinds,
        Some(set) if set.is_empty()
    );
    if !expects_dead_code && !opts_into_dead_code {
        let set = fixture
            .config
            .suppressed_issue_kinds
            .get_or_insert_with(Default::default);
        for kind in dead {
            set.insert((*kind).to_string());
        }
    }
}

fn run_cursor_fixture(path: &str, content: &str, fixture: &ParsedFixture, cursor: &Cursor) {
    let actual = with_fixture_session(&file_refs(fixture), &fixture.config, |session, ws| {
        let file = ws.dir.join(&fixture.files[cursor.file].0);
        // An editor analyzes the file it has open even when PSR-4 discovery
        // would otherwise leave it unloaded.
        let mut analyzed = ws.analyzed.clone();
        if !analyzed.contains(&file) {
            analyzed.push(file.clone());
        }
        session.analyze_paths(&analyzed, &BatchOptions::new().without_symbols());
        run_cursor_query(session, ws, &file.to_string_lossy(), cursor)
    });

    if update_requested() {
        rewrite_expect_section(path, content, &actual);
        return;
    }

    if actual != cursor.expected {
        let desc = fixture
            .description
            .as_deref()
            .map(|d| format!("\n\nDescription: {d}"))
            .unwrap_or_default();
        panic!(
            "fixture {path} FAILED:{desc}\n\nExpected:\n{}\n\nActual:\n{}\n\nTo update: {UPDATE_HINT}",
            fmt_lines(&cursor.expected),
            fmt_lines(&actual),
        );
    }
}

fn run_cursor_query(
    session: &mut AnalysisSession,
    ws: &FixtureWorkspace,
    file: &str,
    cursor: &Cursor,
) -> Vec<String> {
    let not_found = || vec![format!("error: {:?}", crate::SymbolLookupError::NotFound)];
    match cursor.query {
        CursorQuery::Symbol => {
            let whole_file = whole_file_symbol_at(session, file, cursor.offset);
            let targeted = session.symbol_at(file, cursor.offset);
            let (whole_file, targeted) = (
                whole_file.as_ref().map(fmt_symbol),
                targeted.as_ref().map(fmt_symbol),
            );
            assert_eq!(
                whole_file, targeted,
                "FileAnalysis::symbol_at and AnalysisSession::symbol_at disagree"
            );
            whole_file.unwrap_or_else(not_found)
        }
        CursorQuery::Definition => match session.name_at(file, cursor.offset) {
            Some(name) => match session.definition_of_cached(&name) {
                Ok(def) => vec![ws.fmt_location(&def)],
                Err(e) => vec![format!("error: {e:?}")],
            },
            None => not_found(),
        },
        CursorQuery::References {
            include_declaration,
            includes,
        } => {
            let Some(name) = session.name_at(file, cursor.offset) else {
                return not_found();
            };
            let files: Vec<Arc<str>> = ws
                .project_files
                .iter()
                .map(|p| Arc::from(p.to_string_lossy().as_ref()))
                .collect();
            let mut refs = session
                .indexed_references_to(&name, &files, include_declaration, includes, &|| false)
                .expect("uncancelled references query");
            refs.sort_by_key(|(file, range)| (file.clone(), range.start, range.end));
            refs.iter()
                .map(|(file, range)| ws.fmt_range(file, range))
                .collect()
        }
    }
}

/// `symbol_at` on a whole-file analysis with retained symbols, the path an
/// editor takes when it keeps the file's `FileAnalysis` around.
fn whole_file_symbol_at(
    session: &mut AnalysisSession,
    file: &str,
    offset: u32,
) -> Option<crate::ResolvedSymbol> {
    use crate::db::MirDatabase;
    let (text, parsed) = {
        let view = session.db_view();
        let db = view.db();
        let sf = db.lookup_source_file(file)?;
        let prepared = crate::db::prepare_analysis_file(db, sf);
        (prepared.text.clone(), prepared.parsed.0.clone())
    };
    crate::FileAnalyzer::new(session)
        .analyze(Arc::from(file), &text, &parsed.program, &parsed.source_map)
        .symbol_at(offset)
        .cloned()
}

fn fmt_symbol(symbol: &crate::ResolvedSymbol) -> Vec<String> {
    vec![
        format!("kind: {}", fmt_reference_kind(&symbol.kind)),
        format!("type: {}", symbol.resolved_type),
    ]
}

fn fmt_reference_kind(kind: &crate::ReferenceKind) -> String {
    use crate::ReferenceKind as K;
    match kind {
        K::Variable(name) => format!("variable ${}", name.trim_start_matches('$')),
        K::MethodCall { class, method } => format!("method call {class}::{method}"),
        K::StaticCall { class, method } => format!("static call {class}::{method}"),
        K::PropertyAccess { class, property } => format!("property {class}::${property}"),
        K::FunctionCall(name) => format!("function call {name}"),
        K::ClassReference(name) => format!("class {name}"),
        K::ConstantAccess { class, constant } => format!("class constant {class}::{constant}"),
        K::GlobalConstant(name) => format!("global constant {name}"),
        K::UseImport(inner) => format!("use import of {}", fmt_reference_kind(inner)),
        K::Receiver => "receiver".to_string(),
    }
}

fn file_refs(fixture: &ParsedFixture) -> Vec<(&str, &str)> {
    fixture
        .files
        .iter()
        .map(|(n, s)| (n.as_str(), s.as_str()))
        .collect()
}

// ---------------------------------------------------------------------------
// Core analyzer runner
// ---------------------------------------------------------------------------

/// Shared on-disk stub-slice cache dir, reused across fixtures, binaries, and
/// runs. Keyed by (path, content_hash, php_version, mir_version) where
/// mir_version covers the stub-collection sources, so it self-invalidates and
/// stale entries are never read.
fn fixture_stub_cache_dir() -> std::path::PathBuf {
    std::env::temp_dir().join("mir-fixture-stub-cache")
}

/// Pool of pre-warmed base sessions reused across "plain" fixtures (no user
/// stubs, no PSR-4/composer). A checked-out session has the stdlib stub index
/// already materialized — the dominant per-fixture cost. The caller analyzes
/// its fixture files, then invalidates them to restore the stubs-only state
/// before returning the session, so each reuse is independent. Keyed by PHP
/// version; entries with different versions never mix. Opt out with
/// `MIR_TEST_NO_SESSION_POOL=1` (the fresh-session path then runs per fixture).
static SESSION_POOL: Mutex<Vec<(PhpVersion, AnalysisSession)>> = Mutex::new(Vec::new());

fn session_pool_enabled() -> bool {
    std::env::var_os("MIR_TEST_NO_SESSION_POOL").is_none()
        && std::env::var_os("MIR_TEST_NO_STUB_CACHE").is_none()
}

fn checkout_base_session(version: PhpVersion) -> AnalysisSession {
    {
        let mut pool = SESSION_POOL.lock();
        if let Some(pos) = pool.iter().position(|(v, _)| *v == version) {
            return pool.swap_remove(pos).1;
        }
    }
    AnalysisSession::new(version).with_cache_dir(&fixture_stub_cache_dir())
}

fn return_base_session(version: PhpVersion, session: AnalysisSession) {
    SESSION_POOL.lock().push((version, session));
}

/// A fixture's files written to a temp directory.
struct FixtureWorkspace {
    dir: PathBuf,
    /// PHP files outside the configured stubs.
    project_files: Vec<PathBuf>,
    /// Project files to pass to `analyze_paths`; PSR-4-mapped ones are left
    /// for lazy discovery.
    analyzed: Vec<PathBuf>,
}

impl FixtureWorkspace {
    /// `file` relative to the fixture root; paths outside it (stubs) as-is.
    fn display_path(&self, file: &str) -> String {
        let dir = self.dir.to_string_lossy();
        file.strip_prefix(dir.as_ref())
            .map_or(file, |rel| rel.trim_start_matches(['/', '\\']))
            .replace('\\', "/")
    }

    fn fmt_location(&self, loc: &mir_types::Location) -> String {
        format!(
            "{}@{}:{}-{}:{}",
            self.display_path(&loc.file),
            loc.line,
            loc.col_start,
            loc.line_end,
            loc.col_end
        )
    }

    fn fmt_range(&self, file: &str, range: &crate::Range) -> String {
        format!(
            "{}@{}:{}-{}:{}",
            self.display_path(file),
            range.start.line,
            range.start.column,
            range.end.line,
            range.end.column
        )
    }
}

/// Write `files` to a fresh temp directory, then call `run` with a session
/// configured for them (PHP version, user stubs, PSR-4 map).
fn with_fixture_session<R>(
    files: &[(&str, &str)],
    config: &FixtureConfig,
    run: impl FnOnce(&mut AnalysisSession, &FixtureWorkspace) -> R,
) -> R {
    // The pid disambiguates concurrent nextest processes: COUNTER is
    // process-local and resets to 0 in each, so without it they'd all share
    // `mir_fixture_0/` and clobber each other's `test.php`.
    let id = COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_dir = std::env::temp_dir().join(format!("mir_fixture_{}_{id}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir)
        .unwrap_or_else(|e| panic!("failed to create temp dir {}: {e}", tmp_dir.display()));

    let paths: Vec<PathBuf> = files
        .iter()
        .map(|(name, src)| {
            let path = tmp_dir.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .unwrap_or_else(|e| panic!("failed to create dir for {name}: {e}"));
            }
            std::fs::write(&path, src).unwrap_or_else(|e| panic!("failed to write {name}: {e}"));
            path
        })
        .collect();

    // Resolve the requested PHP version (defaulting to LATEST), user stubs, and
    // PSR-4/composer mapping, plus the set of files to analyze. Session
    // construction is deferred until after we know whether this fixture can
    // reuse a pooled base session.
    let version = config.php_version.unwrap_or(PhpVersion::LATEST);

    let stub_files: Vec<PathBuf> = config.stub_files.iter().map(|f| tmp_dir.join(f)).collect();
    let stub_dirs: Vec<PathBuf> = config.stub_dirs.iter().map(|d| tmp_dir.join(d)).collect();
    let stub_file_set: HashSet<PathBuf> = stub_files.iter().cloned().collect();
    let project_files: Vec<PathBuf> = php_files_only(&paths)
        .into_iter()
        .filter(|p| !stub_file_set.contains(p) && !stub_dirs.iter().any(|d| p.starts_with(d)))
        .collect();

    let has_composer = files.iter().any(|(name, _)| *name == "composer.json");
    let psr4 = has_composer
        .then(|| crate::composer::Psr4Map::from_composer(&tmp_dir).ok())
        .flatten()
        .map(Arc::new);
    let analyzed: Vec<PathBuf> = match &psr4 {
        Some(map) => {
            let psr4_files: HashSet<PathBuf> = map.project_files().into_iter().collect();
            project_files
                .iter()
                .filter(|p| !psr4_files.contains(*p))
                .cloned()
                .collect()
        }
        None => project_files.clone(),
    };

    let ws = FixtureWorkspace {
        dir: tmp_dir,
        project_files,
        analyzed,
    };

    // The ~96% of fixtures with no user stubs and no PSR-4/composer need nothing
    // in their session beyond the stdlib stubs, which are identical across all
    // such fixtures. Reuse a pooled base session (stub index already
    // materialized) and reset it afterwards by invalidating the fixture's files,
    // amortizing the one-time stub-index build. Fixtures with custom stubs or a
    // composer map get a fresh, isolated session.
    let reusable = stub_files.is_empty()
        && stub_dirs.is_empty()
        && !has_composer
        && !config.memoize_method_call_results
        && session_pool_enabled();

    let result = if reusable {
        let mut session = checkout_base_session(version);
        let result = run(&mut session, &ws);
        for p in &ws.analyzed {
            session.invalidate_file(&p.to_string_lossy());
        }
        return_base_session(version, session);
        result
    } else {
        let mut session = AnalysisSession::new(version)
            .with_memoize_method_call_results(config.memoize_method_call_results);
        if std::env::var_os("MIR_TEST_NO_STUB_CACHE").is_none() {
            session = session.with_cache_dir(&fixture_stub_cache_dir());
        }
        if !stub_files.is_empty() || !stub_dirs.is_empty() {
            session = session.with_user_stubs(stub_files, stub_dirs);
        }
        if let Some(map) = psr4 {
            session = session.with_psr4(map);
        }
        run(&mut session, &ws)
    };
    std::fs::remove_dir_all(&ws.dir).ok();
    result
}

fn run_analyzer(files: &[(&str, &str)], config: &FixtureConfig) -> Vec<Issue> {
    let mut opts = BatchOptions::new().without_symbols();
    if let Some(explicit) = &config.suppressed_issue_kinds {
        opts.suppressed_issue_kinds = explicit.clone();
    }
    let dead_code_enabled = crate::batch::dead_code_issue_kinds()
        .iter()
        .any(|k| !opts.suppressed_issue_kinds.contains(*k));

    with_fixture_session(files, config, |session, ws| {
        let tmp_dir_str = ws.dir.to_string_lossy();
        session
            .analyze_paths(&ws.analyzed, &opts)
            .issues
            .into_iter()
            .filter(|i| !i.suppressed)
            // When dead-code analysis runs, the analyzer walks the entire
            // codebase including stubs. Filter to issues from the temp directory
            // only so stub-side false positives don't pollute fixture output.
            .filter(|i| {
                !dead_code_enabled || i.location.file.as_ref().starts_with(tmp_dir_str.as_ref())
            })
            .collect()
    })
}

fn php_files_only(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .filter(|p| p.extension().map(|e| e == "php").unwrap_or(false))
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// Fixture assertion
// ---------------------------------------------------------------------------

fn assert_fixture(path: &str, fixture: &ParsedFixture, actual: &[Issue]) {
    let failures = issue_failures(&fixture.expected, actual, fixture.is_multi);
    if !failures.is_empty() {
        panic!(
            "fixture {path} FAILED:{}\n{}\n\nTo fix: ensure all expected issues have @line:col-line_end:col_end locations, then run: {UPDATE_HINT}\n\nAll actual issues:\n{}",
            fmt_description(fixture),
            failures.join("\n"),
            fmt_issues(actual, fixture.is_multi)
        );
    }
}

fn fmt_description(fixture: &ParsedFixture) -> String {
    fixture
        .description
        .as_deref()
        .map(|d| format!("\n\nDescription: {d}"))
        .unwrap_or_default()
}

/// `MISSING`/`UNEXPECTED` lines for `actual` against `expected`.
fn issue_failures(expected: &[ExpectedIssue], actual: &[Issue], is_multi: bool) -> Vec<String> {
    let mut failures: Vec<String> = Vec::new();

    for exp in expected {
        if exp.line.is_none() || exp.col_start.is_none() {
            failures.push(format!(
                "  MISSING LOCATION  {}: expected issue must include @line:col (e.g., {}@1:1: {})",
                exp.kind_name, exp.kind_name, exp.message
            ));
        }
        if !actual.iter().any(|a| issue_matches(a, exp)) {
            failures.push(format!("  MISSING  {}", fmt_expected(exp, is_multi)));
        }
    }

    for act in actual {
        if !expected.iter().any(|e| issue_matches(act, e)) {
            failures.push(format!("  UNEXPECTED {}", fmt_actual(act, is_multi)));
        }
    }
    failures
}

fn issue_matches(actual: &Issue, expected: &ExpectedIssue) -> bool {
    if actual.kind.name() != expected.kind_name {
        return false;
    }
    if actual.kind.message() != expected.message.as_str() {
        return false;
    }
    if let Some(expected_file) = &expected.file {
        let actual_basename = Path::new(actual.location.file.as_ref())
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        if actual_basename.as_ref() != expected_file.as_str() {
            return false;
        }
    }
    if let Some(line) = expected.line {
        if actual.location.line != line {
            return false;
        }
    }
    if let Some(col) = expected.col_start {
        if actual.location.col_start != col {
            return false;
        }
    }
    if let Some(line_end) = expected.line_end {
        if actual.location.line_end != line_end {
            return false;
        }
    }
    if let Some(col_end) = expected.col_end {
        if actual.location.col_end != col_end {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// UPDATE_FIXTURES rewrite
// ---------------------------------------------------------------------------

/// Sorted `===expect===` lines for `actual`.
fn fmt_expect_lines(actual: &[Issue], is_multi: bool) -> Vec<String> {
    let mut sorted: Vec<&Issue> = actual.iter().collect();
    if is_multi {
        sorted.sort_by_key(|i| {
            let basename = Path::new(i.location.file.as_ref())
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            (
                basename,
                i.location.line,
                i.location.col_start,
                i.kind.name(),
            )
        });
    } else {
        sorted.sort_by_key(|i| (i.location.line, i.location.col_start, i.kind.name()));
    }
    sorted
        .into_iter()
        .map(|i| fmt_actual(i, is_multi))
        .collect()
}

/// Rewrite only the `===expect===` section, preserving everything before it.
fn rewrite_expect_section(path: &str, content: &str, lines: &[String]) {
    let exp_pos = content
        .find(EXPECT_MARKER)
        .expect("fixture missing ===expect===");

    let mut out = content[..exp_pos].to_string();
    out.push_str(EXPECT_MARKER);
    out.push('\n');
    for line in lines {
        out.push_str(line);
        out.push('\n');
    }

    std::fs::write(path, &out).unwrap_or_else(|e| panic!("failed to write fixture {path}: {e}"));
}

/// Edit-fixture expect lines, where an empty section must read `<<none>>`.
fn parse_edit_expect_lines(text: &str, is_multi: bool, path: &str) -> Vec<ExpectedIssue> {
    let lines: Vec<&str> = meaningful_lines(text).collect();
    match lines.as_slice() {
        [] => {
            panic!("fixture {path}: an edit fixture's empty {EXPECT_MARKER} must read {NO_ISSUES}")
        }
        [NO_ISSUES] => Vec::new(),
        _ => {
            assert!(
                !lines.contains(&NO_ISSUES),
                "fixture {path}: {NO_ISSUES} can't be combined with expected issues"
            );
            parse_expect_lines(text, is_multi, path)
        }
    }
}

/// Rewrite both `===expect===` sections of an edit fixture, keeping the edits.
fn rewrite_edit_expect_sections(path: &str, content: &str, before: &[String], after: &[String]) {
    let first = content.find(EXPECT_MARKER).unwrap();
    let edits = content.find(EDIT_PREFIX).unwrap();
    let last = content.rfind(EXPECT_MARKER).unwrap();

    let mut out = content[..first].to_string();
    for (lines, following) in [(before, &content[edits..last]), (after, "")] {
        out.push_str(EXPECT_MARKER);
        out.push('\n');
        let lines = if lines.is_empty() {
            &[NO_ISSUES.to_string()][..]
        } else {
            lines
        };
        for line in lines {
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(following);
    }

    std::fs::write(path, &out).unwrap_or_else(|e| panic!("failed to write fixture {path}: {e}"));
}

// ---------------------------------------------------------------------------
// Assertion helpers (used by inline tests)
// ---------------------------------------------------------------------------

/// Assert that `issues` contains at least one issue with the exact `IssueKind`
/// at `line` and `col_start`.
pub fn assert_issue(issues: &[Issue], kind: IssueKind, line: u32, col_start: u16) {
    let found = issues
        .iter()
        .any(|i| i.kind == kind && i.location.line == line && i.location.col_start == col_start);
    if !found {
        panic!(
            "Expected issue {:?} at line {line}, col {col_start}.\nActual issues:\n{}",
            kind,
            fmt_issues(issues, false),
        );
    }
}

/// Assert that `issues` contains at least one issue whose `kind.name()` equals
/// `kind_name` at `line` and `col_start`.
pub fn assert_issue_kind(issues: &[Issue], kind_name: &str, line: u32, col_start: u16) {
    let found = issues.iter().any(|i| {
        i.kind.name() == kind_name && i.location.line == line && i.location.col_start == col_start
    });
    if !found {
        panic!(
            "Expected issue {kind_name} at line {line}, col {col_start}.\nActual issues:\n{}",
            fmt_issues(issues, false),
        );
    }
}

/// Assert that `issues` contains no issue whose `kind.name()` equals `kind_name`.
pub fn assert_no_issue(issues: &[Issue], kind_name: &str) {
    let found: Vec<_> = issues
        .iter()
        .filter(|i| i.kind.name() == kind_name)
        .collect();
    if !found.is_empty() {
        panic!(
            "Expected no {kind_name} issues, but found:\n{}",
            fmt_issues(&found.into_iter().cloned().collect::<Vec<_>>(), false),
        );
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

fn fmt_expected(exp: &ExpectedIssue, is_multi: bool) -> String {
    let kind_with_loc = if let (Some(line), Some(col)) = (exp.line, exp.col_start) {
        if let (Some(le), Some(ce)) = (exp.line_end, exp.col_end) {
            format!("{}@{}:{}-{}:{}", exp.kind_name, line, col, le, ce)
        } else {
            format!("{}@{}:{}", exp.kind_name, line, col)
        }
    } else {
        exp.kind_name.clone()
    };

    if is_multi {
        if let Some(f) = &exp.file {
            return format!("{}: {}: {}", f, kind_with_loc, exp.message);
        }
    }
    format!("{}: {}", kind_with_loc, exp.message)
}

fn fmt_actual(act: &Issue, is_multi: bool) -> String {
    if is_multi {
        let basename = Path::new(act.location.file.as_ref())
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return format!(
            "{}: {}@{}:{}-{}:{}: {}",
            basename,
            act.kind.name(),
            act.location.line,
            act.location.col_start,
            act.location.line_end,
            act.location.col_end,
            act.kind.message()
        );
    }
    format!(
        "{}@{}:{}-{}:{}: {}",
        act.kind.name(),
        act.location.line,
        act.location.col_start,
        act.location.line_end,
        act.location.col_end,
        act.kind.message()
    )
}

fn fmt_issues(issues: &[Issue], is_multi: bool) -> String {
    if issues.is_empty() {
        return "  (none)".to_string();
    }
    issues
        .iter()
        .map(|i| format!("  {}", fmt_actual(i, is_multi)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fmt_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return "  (none)".to_string();
    }
    lines
        .iter()
        .map(|l| format!("  {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Fixture parser validation tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod parser_validation {
    use super::{parse_phpt, CursorQuery, ParsedFixture, ReferenceIncludes};

    fn p(content: &str) -> ParsedFixture {
        parse_phpt(content, "<test>")
    }

    #[test]
    fn annotation_becomes_expected_issue() {
        let f = p("===file===\n<?php\n\nfoo();\n//^^^ K: msg here\n// ^^ Other\n===expect===\n");
        assert_eq!(f.expected.len(), 2);
        let a = &f.expected[0];
        assert_eq!(
            (a.line, a.col_start, a.line_end, a.col_end),
            (Some(3), Some(2), Some(3), Some(5))
        );
        assert_eq!(
            (a.kind_name.as_str(), a.message.as_str()),
            ("K", "msg here")
        );
        let b = &f.expected[1];
        assert_eq!(
            (b.line, b.col_start, b.col_end),
            (Some(3), Some(3), Some(5))
        );
        assert_eq!((b.kind_name.as_str(), b.message.as_str()), ("Other", ""));
    }

    #[test]
    fn annotation_at_column_zero_and_multiline_end() {
        let f =
            p("===file===\n<?php\nfoo();\n//<^^^ K\nfoo(\n// ^ +1:2 M: multi\n);\n===expect===\n");
        let a = &f.expected[0];
        assert_eq!(
            (a.line, a.col_start, a.line_end, a.col_end),
            (Some(2), Some(0), Some(2), Some(3))
        );
        let b = &f.expected[1];
        assert_eq!(
            (b.line, b.col_start, b.line_end, b.col_end),
            (Some(4), Some(3), Some(6), Some(2))
        );
        assert_eq!((b.kind_name.as_str(), b.message.as_str()), ("M", "multi"));
    }

    #[test]
    fn annotation_in_multi_file_names_its_file() {
        let f = p(
            "===file:A.php===\n<?php\nfoo();\n// ^^^ K\n===file:B.php===\n<?php\n===expect===\n",
        );
        assert_eq!(f.expected[0].file.as_deref(), Some("A.php"));
    }

    #[test]
    #[should_panic(expected = "has no source line above it")]
    fn annotation_without_source_line() {
        p("===file===\n// ^^^ K\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===file=== must appear at most once")]
    fn duplicate_bare_file_marker() {
        p("===file===\n<?php\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "cannot mix ===file=== and ===file:name===")]
    fn mixed_bare_and_named_markers() {
        p("===file===\n<?php\n===file:Other.php===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===config=== must appear at most once")]
    fn duplicate_config_section() {
        p("===config===\nsuppress=Foo\n===config===\nsuppress=Bar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "unknown config key")]
    fn unknown_config_key() {
        p("===config===\nfoo=bar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "invalid php_version")]
    fn invalid_php_version() {
        p("===config===\nphp_version=banana\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===config=== must appear before the first ===file===")]
    fn config_after_file_marker() {
        p("===file===\n<?php\n===config===\nsuppress=Foo\n===expect===\n");
    }

    #[test]
    fn valid_config_is_accepted() {
        p("===config===\nphp_version=8.1\nsuppress=Foo,Bar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===description=== must appear at most once")]
    fn duplicate_description_section() {
        p("===description===\nfoo\n===description===\nbar\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===description=== must appear before the first ===file===")]
    fn description_after_file_marker() {
        p("===file===\n<?php\n===description===\nfoo\n===expect===\n");
    }

    #[test]
    fn valid_description_is_accepted() {
        let f = p("===description===\nChecks null method call.\n===file===\n<?php\n===expect===\n");
        assert_eq!(f.description.as_deref(), Some("Checks null method call."));
    }

    #[test]
    #[should_panic(expected = "===ignore=== must appear at most once")]
    fn duplicate_ignore_marker() {
        p("===ignore===\n===ignore===\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===ignore=== must appear before the first ===file===")]
    fn ignore_after_file_marker() {
        p("===file===\n<?php\n===ignore===\n===expect===\n");
    }

    #[test]
    fn valid_ignore_is_accepted() {
        let f = p("===ignore===\n===file===\n<?php\n===expect===\n");
        assert!(f.description.is_none());
    }

    #[test]
    fn cursor_marker_is_removed_and_located() {
        let f = p("===cursor===\nreferences include_declaration use_imports\n\
                   ===file:a.php===\n<?php\n===file:b.php===\n<?php f<CURSOR>oo();\n\
                   ===expect===\nb.php@1:6-1:9\n");
        let cursor = f.cursor.expect("cursor fixture");
        assert_eq!(
            cursor.query,
            CursorQuery::References {
                include_declaration: true,
                includes: ReferenceIncludes::PlainAndUseImports,
            }
        );
        assert_eq!((cursor.file, cursor.offset), (1, 7));
        assert_eq!(f.files[1].1, "<?php foo();");
        assert_eq!(cursor.expected, ["b.php@1:6-1:9"]);
        assert!(f.expected.is_empty());
    }

    #[test]
    fn config_before_cursor_section_is_accepted() {
        let f = p("===config===\nphp_version=8.1\n===cursor===\nsymbol\n\
                   ===file===\n<?php f<CURSOR>();\n===expect===\n");
        assert_eq!(f.cursor.map(|c| c.query), Some(CursorQuery::Symbol));
    }

    #[test]
    #[should_panic(expected = "===cursor=== needs a <CURSOR> marker")]
    fn cursor_section_without_marker() {
        p("===cursor===\nsymbol\n===file===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "<CURSOR> marker needs a ===cursor=== section")]
    fn marker_without_cursor_section() {
        p("===file===\n<?php f<CURSOR>();\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "<CURSOR> must appear exactly once across all files")]
    fn marker_in_two_files() {
        p("===cursor===\nsymbol\n===file:a.php===\n<?php <CURSOR>\n\
           ===file:b.php===\n<?php <CURSOR>\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "unknown ===cursor=== query")]
    fn unknown_cursor_query() {
        p("===cursor===\ncompletion\n===file===\n<?php <CURSOR>\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "unknown references option")]
    fn unknown_references_option() {
        p("===cursor===\nreferences all\n===file===\n<?php <CURSOR>\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "symbol takes no options")]
    fn symbol_with_options() {
        p("===cursor===\nsymbol include_declaration\n===file===\n<?php <CURSOR>\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "suppress has no effect in a ===cursor=== fixture")]
    fn suppress_in_cursor_fixture() {
        p("===config===\nsuppress=Foo\n===cursor===\nsymbol\n\
           ===file===\n<?php <CURSOR>\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "===cursor=== must appear before the first ===file===")]
    fn cursor_after_file_marker() {
        p("===file===\n<?php <CURSOR>\n===cursor===\nsymbol\n===expect===\n");
    }

    #[test]
    fn edits_are_split_from_files_and_expects_in_order() {
        let f = p(
            "===file:a.php===\n<?php a();\n===file:b.php===\n<?php b();\n\
             ===expect===\na.php: UndefinedFunction@1:6-1:9: Function a() is not defined\n\
             ===edit:b.php===\n<?php b2();\n===edit:a.php===\n<?php a2();\n\
             ===edit:b.php===\n<?php b3();\n===expect===\n<<none>>\n",
        );
        assert_eq!(f.files[0].1, "<?php a();");
        assert_eq!(f.files[1].1, "<?php b();");
        let edits: Vec<_> = f
            .edits
            .iter()
            .map(|(n, t)| (n.as_str(), t.as_str()))
            .collect();
        assert_eq!(
            edits,
            [
                ("b.php", "<?php b2();"),
                ("a.php", "<?php a2();"),
                ("b.php", "<?php b3();")
            ]
        );
        let before: Vec<_> = f
            .expected_before_edits
            .iter()
            .map(|e| (e.file.as_deref(), e.kind_name.as_str()))
            .collect();
        assert_eq!(before, [(Some("a.php"), "UndefinedFunction")]);
        assert!(f.expected.is_empty());
    }

    #[test]
    #[should_panic(expected = "an edit fixture's empty ===expect=== must read <<none>>")]
    fn empty_expect_before_edits() {
        p("===file:a.php===\n<?php\n===expect===\n===edit:a.php===\n<?php\n===expect===\n<<none>>\n");
    }

    #[test]
    #[should_panic(expected = "an edit fixture's empty ===expect=== must read <<none>>")]
    fn empty_expect_after_edits() {
        p("===file:a.php===\n<?php\n===expect===\n<<none>>\n===edit:a.php===\n<?php\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "<<none>> can't be combined with expected issues")]
    fn no_issues_placeholder_with_issues() {
        p("===file:a.php===\n<?php a();\n===expect===\n<<none>>\n\
           a.php: UndefinedFunction@1:6-1:9: Function a() is not defined\n\
           ===edit:a.php===\n<?php\n===expect===\n<<none>>\n");
    }

    #[test]
    fn no_issues_placeholder_expects_nothing() {
        let f = p("===file:a.php===\n<?php\n===expect===\n<<none>>\n\
                   ===edit:a.php===\n<?php\n===expect===\n<<none>>\n");
        assert!(f.expected_before_edits.is_empty() && f.expected.is_empty());
    }

    #[test]
    #[should_panic(expected = "===expect=== must appear twice")]
    fn edit_without_expect_before_edits() {
        p("===file:a.php===\n<?php\n===edit:a.php===\n<?php\n===expect===\n<<none>>\n");
    }

    #[test]
    #[should_panic(expected = "===expect=== must appear exactly once")]
    fn two_expects_without_edits() {
        p("===file:a.php===\n<?php\n===expect===\n===expect===\n");
    }

    #[test]
    #[should_panic(expected = "the first ===expect=== must sit between the file and edit sections")]
    fn expect_between_edits() {
        p(
            "===file:a.php===\n<?php\n===edit:a.php===\n<?php\n===expect===\n<<none>>\n\
           ===edit:a.php===\n<?php\n===expect===\n<<none>>\n",
        );
    }

    #[test]
    #[should_panic(expected = "the first ===expect=== must sit between the file and edit sections")]
    fn expect_before_file_section() {
        p(
            "===file:a.php===\n<?php\n===expect===\n<<none>>\n===file:b.php===\n<?php\n\
           ===edit:a.php===\n<?php\n===expect===\n<<none>>\n",
        );
    }

    #[test]
    #[should_panic(expected = "===edit:name=== must follow every file section")]
    fn edit_before_file_section() {
        p(
            "===file:a.php===\n<?php\n===expect===\n<<none>>\n===edit:a.php===\n<?php\n\
           ===file:b.php===\n<?php\n===expect===\n<<none>>\n",
        );
    }

    #[test]
    #[should_panic(expected = "===edit:name=== needs ===file:name=== sections")]
    fn edit_with_bare_file() {
        p("===file===\n<?php\n===expect===\n<<none>>\n===edit:test.php===\n<?php\n===expect===\n<<none>>\n");
    }

    #[test]
    #[should_panic(expected = "===edit:c.php=== names no ===file:c.php=== section")]
    fn edit_of_undeclared_file() {
        p("===file:a.php===\n<?php\n===expect===\n<<none>>\n===edit:c.php===\n<?php\n===expect===\n<<none>>\n");
    }

    #[test]
    #[should_panic(expected = "===edit:name=== can't be combined with ===cursor===")]
    fn edit_in_cursor_fixture() {
        p(
            "===cursor===\nsymbol\n===file:a.php===\n<?php <CURSOR>\n===expect===\n<<none>>\n\
           ===edit:a.php===\n<?php\n===expect===\n<<none>>\n",
        );
    }
}
