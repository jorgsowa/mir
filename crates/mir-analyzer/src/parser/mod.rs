pub mod docblock;
pub(crate) mod type_from_hint;

use std::borrow::Cow;
use std::sync::Arc;

use php_ast::Span;
use php_rs_parser::diagnostics::{ForbiddenRule, Phase};

pub use docblock::{DocblockParser, ParsedDocblock};
pub use type_from_hint::{type_from_hint, type_from_hint_owned};

// ---------------------------------------------------------------------------
// Parse-error → Issue conversion
// ---------------------------------------------------------------------------

/// Convert a parser diagnostic to a [`mir_issues::Issue`], using the source
/// and source map to derive a precise location.
pub(crate) fn parse_error_to_issue(
    err: &php_rs_parser::diagnostics::ParseError,
    file: &Arc<str>,
    source: &str,
    parsed: &php_rs_parser::ParseResult,
) -> mir_issues::Issue {
    let span = err.span();
    let source_map = &parsed.source_map;
    let (line, col_start) = crate::diagnostics::offset_to_line_col(source, span.start, source_map);
    let (line_end, col_end) = crate::diagnostics::offset_to_line_col(source, span.end, source_map);

    let message = php_message(err, &parsed.program).unwrap_or_else(|| err.to_string());
    let code = err.code().to_owned();
    let rule = err.forbidden_rule();
    let kind = if rule == Some(ForbiddenRule::ParentWithoutParentClass) {
        mir_issues::IssueKind::ParentNotFound
    } else if rule.map(ForbiddenRule::phase) == Some(Phase::Compile) {
        mir_issues::IssueKind::CompileError { message, code }
    } else {
        mir_issues::IssueKind::ParseError { message, code }
    };
    mir_issues::Issue::new(
        kind,
        mir_issues::Location {
            file: file.clone(),
            line,
            line_end,
            col_start,
            col_end,
        },
    )
}

/// PHP's wording for parser diagnostics whose text lacks the property PHP names.
fn php_message(
    err: &php_rs_parser::diagnostics::ParseError,
    program: &php_ast::owned::Program,
) -> Option<String> {
    let message: fn(&str, &str) -> String = match err.forbidden_rule()? {
        ForbiddenRule::UntypedReadonlyProperty => {
            |class, prop| format!("Readonly property {class}::${prop} must have type")
        }
        ForbiddenRule::StaticReadonlyProperty => {
            |class, prop| format!("Static property {class}::${prop} cannot be readonly")
        }
        ForbiddenRule::ReadonlyPropertyHooks => {
            return Some("Hooked properties cannot be readonly".to_string())
        }
        _ => return None,
    };
    let mut finder = PropertyAt {
        offset: err.span().start,
        namespace: String::new(),
        classes: Vec::new(),
        found: None,
    };
    for stmt in &program.stmts {
        if php_ast::owned::visitor::OwnedVisitor::visit_stmt(&mut finder, stmt).is_break() {
            break;
        }
    }
    let (class, property) = finder.found?;
    Some(message(&class, &property))
}

/// Finds the class-like and name of the property or promoted parameter that starts at `offset`.
struct PropertyAt {
    offset: u32,
    namespace: String,
    classes: Vec<String>,
    found: Option<(String, String)>,
}

impl PropertyAt {
    fn in_class(&mut self, name: &str, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        let name = if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}\\{name}", self.namespace)
        };
        self.classes.push(name);
        let flow = crate::recursion::walk_stmt(self, stmt);
        self.classes.pop();
        flow
    }

    fn found(&mut self, span: Span, name: &php_ast::owned::Ident) -> std::ops::ControlFlow<()> {
        if span.start > self.offset || self.offset >= span.end {
            return std::ops::ControlFlow::Continue(());
        }
        let class = self.classes.last().cloned().unwrap_or_default();
        self.found = Some((class, name.as_str().unwrap_or_default().to_string()));
        std::ops::ControlFlow::Break(())
    }
}

impl php_ast::owned::visitor::OwnedVisitor for PropertyAt {
    type Break = ();

    fn visit_stmt(&mut self, stmt: &php_ast::owned::Stmt) -> std::ops::ControlFlow<()> {
        use php_ast::owned::{NamespaceBody, StmtKind};
        // An unbraced `namespace X;` ends before the code it applies to.
        let unbraced_namespace = matches!(&stmt.kind, StmtKind::Namespace(ns) if !matches!(ns.body, NamespaceBody::Braced(_)));
        if stmt.span.end <= self.offset && !unbraced_namespace {
            return std::ops::ControlFlow::Continue(());
        }
        match &stmt.kind {
            StmtKind::Namespace(ns) => {
                let name = ns
                    .name
                    .as_ref()
                    .map(|n| name_to_string_owned(n).into_owned())
                    .unwrap_or_default();
                if let NamespaceBody::Braced(_) = &ns.body {
                    let outer = std::mem::replace(&mut self.namespace, name);
                    let flow = crate::recursion::walk_stmt(self, stmt);
                    self.namespace = outer;
                    return flow;
                }
                self.namespace = name;
            }
            StmtKind::Class(c) => {
                let name = c.name.as_ref().and_then(|n| n.as_str()).unwrap_or_default();
                return self.in_class(name, stmt);
            }
            StmtKind::Trait(d) => return self.in_class(d.name.as_str().unwrap_or_default(), stmt),
            _ => {}
        }
        crate::recursion::walk_stmt(self, stmt)
    }

    fn visit_class_member(
        &mut self,
        member: &php_ast::owned::ClassMember,
    ) -> std::ops::ControlFlow<()> {
        use php_ast::owned::ClassMemberKind;
        match &member.kind {
            ClassMemberKind::Property(p) => self.found(member.span, &p.name)?,
            ClassMemberKind::Method(m) => {
                for param in m.params.iter() {
                    self.found(param.span, &param.name)?;
                }
            }
            _ => {}
        }
        php_ast::owned::visitor::walk_owned_class_member(self, member)
    }

    fn visit_expr(&mut self, expr: &php_ast::owned::Expr) -> std::ops::ControlFlow<()> {
        if let php_ast::owned::ExprKind::AnonymousClass(_) = &expr.kind {
            self.classes.push("class@anonymous".to_string());
            let flow = crate::recursion::walk_expr(self, expr);
            self.classes.pop();
            return flow;
        }
        crate::recursion::walk_expr(self, expr)
    }
}

/// Compile-time rules the parser checks that the analyzer reports itself with a
/// dedicated issue kind; the parser's diagnostic is dropped as a duplicate.
const ANALYZER_COVERED_RULES: &[ForbiddenRule] = &[
    ForbiddenRule::ToStringReturnType,
    ForbiddenRule::MissingReturnValue,
    ForbiddenRule::VoidReturnValue,
    ForbiddenRule::NeverReturn,
    ForbiddenRule::ReadonlyPropertyDefault,
];

fn is_analyzer_covered_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    err.forbidden_rule()
        .is_some_and(|rule| ANALYZER_COVERED_RULES.contains(&rule))
}

/// Parser diagnostics that are dropped from the issue stream and never block
/// analysis: the ones the analyzer reports itself.
pub(crate) fn is_redundant_parse_error(err: &php_rs_parser::diagnostics::ParseError) -> bool {
    is_analyzer_covered_error(err)
}

/// Whether `errors` leave a recovery guess in the AST, which blocks body
/// analysis. `Forbidden` and `VersionTooLow` are raised on a fully built node.
pub(crate) fn blocks_body_analysis(errors: &[php_rs_parser::diagnostics::ParseError]) -> bool {
    errors.iter().any(|err| err.leaves_ast_incomplete())
}

/// Whether `errors` keep the definition slice out of the parse and stub caches.
/// The parser's diagnostics are cached with the slice, so only an incomplete
/// AST blocks: `Forbidden` and `VersionTooLow` files (bundled stubs declaring
/// per-version variants, polyfills behind `PHP_VERSION_ID` checks) are cached.
pub(crate) fn parse_errors_block_cache(errors: &[php_rs_parser::diagnostics::ParseError]) -> bool {
    blocks_body_analysis(errors)
}

// ---------------------------------------------------------------------------
// Source location helpers
// ---------------------------------------------------------------------------

/// Extract the exact source text covered by a span.
pub(crate) fn span_text(src: &str, span: Span) -> Option<String> {
    if span.start >= span.end {
        return None;
    }
    let s = span.start as usize;
    let e = (span.end as usize).min(src.len());
    src.get(s..e)
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Offset of the first token after the `#[...]` groups (and comments) that open a declaration span.
pub(crate) fn skip_leading_attributes(src: &str, offset: u32) -> u32 {
    let bytes = src.as_bytes();
    let mut i = offset as usize;
    if !src.get(i..).is_some_and(|s| s.starts_with("#[")) {
        return offset;
    }
    loop {
        while i < bytes.len() {
            match bytes[i] {
                b if b.is_ascii_whitespace() => i += 1,
                b'/' if bytes.get(i + 1) == Some(&b'*') => {
                    i = src[i + 2..]
                        .find("*/")
                        .map_or(bytes.len(), |p| i + 2 + p + 2);
                }
                b'#' if bytes.get(i + 1) != Some(&b'[') => i = skip_line(bytes, i),
                b'/' if bytes.get(i + 1) == Some(&b'/') => i = skip_line(bytes, i),
                _ => break,
            }
        }
        if !src[i..].starts_with("#[") {
            return i as u32;
        }
        i += 2;
        let mut depth = 1u32;
        while i < bytes.len() && depth > 0 {
            match bytes[i] {
                b'[' => depth += 1,
                b']' => depth -= 1,
                quote @ (b'\'' | b'"') => {
                    i += 1;
                    while i < bytes.len() && bytes[i] != quote {
                        i += if bytes[i] == b'\\' { 2 } else { 1 };
                    }
                }
                _ => {}
            }
            i += 1;
        }
        i = i.min(bytes.len());
    }
}

fn skip_line(bytes: &[u8], i: usize) -> usize {
    bytes[i..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |p| i + p + 1)
}

// ---------------------------------------------------------------------------
// Docblock extraction from source text
// ---------------------------------------------------------------------------

/// Scan backwards from `offset` and return the `/** ... */` docblock comment
/// that immediately precedes the token at that position, if any.
///
/// Whitespace and class-level modifier keywords (`final`, `abstract`,
/// `readonly`) between the docblock and the declaration are skipped — the
/// php-rs-parser places `span.start` at the `class`/`interface`/`trait`
/// keyword, after any modifiers.
///
/// A `?>`/inline-HTML/`<?php` block boundary is also skipped: a Yii2-style
/// view template routinely writes `/** @var Post $model */` immediately
/// before closing the PHP block, then reopens it later to actually use the
/// variable (php-lsp#235). The docblock is textually adjacent to the
/// declaration in every sense that matters here — nothing but tag
/// punctuation and inert markup sits between them.
pub(crate) fn find_preceding_docblock(source: &str, offset: u32) -> Option<&str> {
    let offset = (offset as usize).min(source.len());
    if offset == 0 {
        return None;
    }
    let mut trimmed = source[..offset].trim_end();
    loop {
        let after_ws = trimmed.trim_end();

        if let Some(before_tag) = strip_trailing_reopen_tag(after_ws) {
            let before_html = before_tag.trim_end();
            trimmed = match before_html.rfind("?>") {
                Some(close_idx) => &before_html[..close_idx],
                // A re-opening tag with nothing before it to close — this is
                // the file's very first `<?php`, not a split block boundary.
                None => before_html,
            };
            continue;
        }

        // Strip trailing modifier keywords like `final` or `abstract readonly`.
        let last_word_start = after_ws
            .char_indices()
            .rfind(|(_, c)| !c.is_ascii_alphabetic())
            .map(|(i, c)| i + c.len_utf8())
            .unwrap_or(0);
        let word = &after_ws[last_word_start..];
        if matches!(word, "final" | "abstract" | "readonly") {
            trimmed = &after_ws[..last_word_start];
        } else {
            trimmed = after_ws;
            break;
        }
    }
    if !trimmed.ends_with("*/") {
        return None;
    }
    let end = trimmed.rfind("*/")?;
    // Prefer /** docblocks; fall back to /* for inline @var annotations (e.g. Yii2 view files).
    let start = trimmed[..end]
        .rfind("/**")
        .or_else(|| trimmed[..end].rfind("/*"))?;
    Some(&trimmed[start..end + 2])
}

/// A closure's docblock: PHP takes the last one before `function` / `fn`, which may follow its attributes.
pub(crate) fn closure_docblock<'s>(
    source: &'s str,
    span_start: u32,
    attributes: &[php_ast::owned::Attribute],
) -> Option<&'s str> {
    attributes
        .last()
        .and_then(|attr| {
            // The attribute span may stop before the group's `,` / `]`.
            let rest = source.get(attr.span.end as usize..)?.trim_start();
            let rest = rest.strip_prefix(',').unwrap_or(rest).trim_start();
            let rest = rest.strip_prefix(']').unwrap_or(rest).trim_start();
            if !rest.starts_with("/*") {
                return None;
            }
            Some(&rest[..rest.find("*/")? + 2])
        })
        .or_else(|| find_preceding_docblock(source, span_start))
}

/// Strip a trailing PHP re-opening tag (`<?php`, `<?=`, or short-open `<?`),
/// returning the text before it. Used by [`find_preceding_docblock`] to look
/// past a `?> ... <?php` gap for the docblock that precedes it.
fn strip_trailing_reopen_tag(s: &str) -> Option<&str> {
    for tag in ["<?php", "<?=", "<?"] {
        if let Some(stripped) = s.strip_suffix(tag) {
            return Some(stripped);
        }
    }
    None
}

/// Gives each class member and parameter the docblock PHP reads for it. The parser looks for it
/// only before the first `#[`, so one written after the attributes went to the next member.
pub(crate) fn attach_docblocks_after_attributes(parsed: &mut php_rs_parser::ParseResult) {
    use php_ast::owned::fold::{
        fold_owned_class_member, fold_owned_enum_member, fold_owned_param, OwnedFold,
    };
    use php_ast::owned::{
        Attribute, ClassMember, ClassMemberKind, Comment, EnumMember, EnumMemberKind, Param,
        Program,
    };

    struct Attach<'s> {
        source: &'s str,
        prev_end: u32,
    }

    impl Attach<'_> {
        fn fix(&self, floor: u32, doc: &mut Option<Comment>, attrs: &[Attribute], name_start: u32) {
            if doc.as_ref().is_some_and(|d| d.span.start < floor) {
                *doc = None;
            }
            let Some(last_attr) = attrs.last() else {
                return;
            };
            let from = last_attr.span.end as usize;
            let Some(between) = self.source.get(from..name_start as usize) else {
                return;
            };
            let Some(open) = between.rfind("/**") else {
                return;
            };
            let Some(len) = between[open..].find("*/") else {
                return;
            };
            let start = (from + open) as u32;
            let text = &between[open..open + len + 2];
            *doc = Some(Comment {
                kind: php_ast::ast::CommentKind::Doc,
                text: text.into(),
                span: Span::new(start, start + text.len() as u32),
            });
        }
    }

    impl OwnedFold for Attach<'_> {
        fn fold_class_member(&mut self, member: ClassMember) -> ClassMember {
            let floor = self.prev_end;
            let mut member = fold_owned_class_member(self, member);
            match &mut member.kind {
                ClassMemberKind::Property(p) => {
                    self.fix(floor, &mut p.doc_comment, &p.attributes, p.name.span.start)
                }
                ClassMemberKind::Method(m) => {
                    self.fix(floor, &mut m.doc_comment, &m.attributes, m.name.span.start)
                }
                ClassMemberKind::ClassConst(c) => {
                    self.fix(floor, &mut c.doc_comment, &c.attributes, c.name.span.start)
                }
                _ => {}
            }
            self.prev_end = member.span.end;
            member
        }

        fn fold_enum_member(&mut self, member: EnumMember) -> EnumMember {
            let floor = self.prev_end;
            let mut member = fold_owned_enum_member(self, member);
            match &mut member.kind {
                EnumMemberKind::Case(c) => {
                    self.fix(floor, &mut c.doc_comment, &c.attributes, c.name.span.start)
                }
                EnumMemberKind::Property(p) => {
                    self.fix(floor, &mut p.doc_comment, &p.attributes, p.name.span.start)
                }
                EnumMemberKind::Method(m) => {
                    self.fix(floor, &mut m.doc_comment, &m.attributes, m.name.span.start)
                }
                EnumMemberKind::ClassConst(c) => {
                    self.fix(floor, &mut c.doc_comment, &c.attributes, c.name.span.start)
                }
                _ => {}
            }
            self.prev_end = member.span.end;
            member
        }

        fn fold_param(&mut self, param: Param) -> Param {
            let floor = self.prev_end;
            let mut param = fold_owned_param(self, param);
            self.fix(
                floor,
                &mut param.doc_comment,
                &param.attributes,
                param.name.span.start,
            );
            self.prev_end = param.span.end;
            param
        }
    }

    let source = parsed.source.as_str();
    let docblock_after_attribute = source
        .match_indices("/**")
        .any(|(i, _)| source[..i].trim_end().ends_with(']'));
    if !docblock_after_attribute {
        return;
    }
    let empty = Program {
        stmts: Box::new([]),
        span: parsed.program.span,
    };
    let program = std::mem::replace(&mut parsed.program, empty);
    parsed.program = Attach {
        source: &parsed.source,
        prev_end: 0,
    }
    .fold_program(program);
}

// ---------------------------------------------------------------------------
// Name resolution helper — join Name parts to a string
// ---------------------------------------------------------------------------

/// A relative name keeps its `namespace\` prefix so resolution skips `use` imports.
pub(crate) fn name_to_string<'src>(name: &php_ast::ast::Name<'_, 'src>) -> Cow<'src, str> {
    if name.kind() == php_ast::ast::NameKind::Relative {
        return Cow::Owned(format!("namespace\\{}", name.join_parts()));
    }
    name.to_string_repr()
}

/// Same as [`name_to_string`] but for the owned (lifetime-free) AST.
pub(crate) fn name_to_string_owned(name: &php_ast::owned::Name) -> Cow<'_, str> {
    let fully_qualified = name.kind == php_ast::ast::NameKind::FullyQualified;
    let relative = name.kind == php_ast::ast::NameKind::Relative;
    if let [single] = &*name.parts {
        if !fully_qualified && !relative {
            return Cow::Borrowed(single);
        }
    }
    let len = name.parts.iter().map(|p| p.len() + 1).sum::<usize>();
    let mut out = String::with_capacity(len + 9);
    if relative {
        out.push_str("namespace");
    }
    for (i, part) in name.parts.iter().enumerate() {
        if fully_qualified || relative || i > 0 {
            out.push('\\');
        }
        out.push_str(part);
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod ast_incomplete_tests {
    use super::{blocks_body_analysis, parse_errors_block_cache};

    fn blocks(src: &str) -> bool {
        let parsed = php_rs_parser::parse(src);
        assert!(!parsed.errors.is_empty());
        parse_errors_block_cache(&parsed.errors)
    }

    #[test]
    fn redeclared_method_leaves_ast_complete() {
        assert!(!blocks(
            "<?php\nclass A { function f() {} function f() {} }\n"
        ));
    }

    #[test]
    fn reserved_function_name_leaves_ast_complete() {
        assert!(!blocks("<?php\nfunction exit() {}\n"));
    }

    #[test]
    fn version_too_low_leaves_ast_complete() {
        let mut options = php_rs_parser::ParseOptions::default();
        options.version = php_ast::PhpVersion::Php74;
        let parsed = php_rs_parser::parse_with("<?php\n$x = match (1) { 1 => 2 };\n", &options);
        assert!(!parsed.errors.is_empty());
        assert!(!blocks_body_analysis(&parsed.errors));
        assert!(!parse_errors_block_cache(&parsed.errors));
    }

    #[test]
    fn syntax_error_leaves_ast_incomplete() {
        assert!(blocks("<?php\nfunction f( {}\n"));
    }

    /// The depth diagnostic must block caching on its own, so a slice built
    /// from a truncated AST is never cached.
    #[test]
    fn expression_depth_limit_leaves_ast_incomplete() {
        let src = format!("<?php\n$x = {}1{};\n", "(".repeat(1000), ")".repeat(1000));
        let parsed = php_rs_parser::parse(&src);
        let depth_err = parsed
            .errors
            .iter()
            .find(|e| {
                matches!(
                    e,
                    php_rs_parser::diagnostics::ParseError::NestingTooDeep { .. }
                )
            })
            .expect("parser no longer emits the depth-limit diagnostic");
        assert!(depth_err.leaves_ast_incomplete());
    }
}

#[cfg(test)]
mod analyzer_covered_tests {
    use super::{is_redundant_parse_error, ANALYZER_COVERED_RULES};

    /// One source per rule in `ANALYZER_COVERED_RULES`, in the same order.
    const SOURCES: &[&str] = &[
        "<?php\nclass A { function __toString(): int { return 1; } }\n",
        "<?php\nfunction f(): int { return; }\n",
        "<?php\nfunction f(): void { return 1; }\n",
        "<?php\nfunction f(): never { return; }\n",
        "<?php\nclass A { public readonly int $x = 1; }\n",
    ];

    #[test]
    fn every_covered_rule_matches_its_parser_diagnostic() {
        assert_eq!(SOURCES.len(), ANALYZER_COVERED_RULES.len());
        for (src, rule) in SOURCES.iter().zip(ANALYZER_COVERED_RULES) {
            let parsed = php_rs_parser::parse(src);
            let rules: Vec<_> = parsed.errors.iter().map(|e| e.forbidden_rule()).collect();
            assert_eq!(rules, [Some(*rule)], "{src:?}: {:?}", parsed.errors);
            assert!(parsed.errors.iter().all(is_redundant_parse_error));
        }
    }

    #[test]
    fn neighbouring_rules_are_not_covered() {
        for src in [
            "<?php\nclass A { function __serialize(): int { return 1; } }\n",
            "<?php\nfunction f($_GET) {}\n",
            "<?php\nfunction f() { unset($this); }\n",
            "<?php\nclass A { function f() { parent::f(); } }\n",
            "<?php\nfunction f() { self::g(); }\n",
            "<?php\nfunction f(): int { yield 1; }\n",
            "<?php\nfunction f($a, $a) {}\n",
            "<?php\n#[Attribute] trait T {}\n",
            "<?php\n#[Deprecated] class A {}\n",
            "<?php\n#[Override] class A {}\n",
            "<?php\nfunction f() { $this = 1; }\n",
        ] {
            let parsed = php_rs_parser::parse(src);
            assert!(
                !parsed.errors.is_empty() && !parsed.errors.iter().any(is_redundant_parse_error),
                "{src:?}: {:?}",
                parsed.errors
            );
        }
    }
}

#[cfg(test)]
mod docblock_lookup_tests {
    use super::find_preceding_docblock;

    #[test]
    fn plain_docblock_immediately_before_offset() {
        let src = "<?php\n/** @var Post $model */\n$x = 1;\n";
        let offset = src.find("$x").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */")
        );
    }

    #[test]
    fn docblock_survives_a_split_php_html_block() {
        let src = "<?php\nclass Post { public string $title = ''; }\n\
                   /** @var Post $model */\n?>\n<div>\n<?php if ($model->title !== ''): ?>\n";
        let offset = src.rfind("if (").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */")
        );
    }

    #[test]
    fn docblock_survives_a_split_block_via_short_echo_reopen() {
        let src = "<?php\n/** @var Post $model */\n?>\n<div>\n<?= $model->title ?>\n";
        let offset = src.rfind("$model->title").unwrap() as u32;
        assert_eq!(
            find_preceding_docblock(src, offset),
            Some("/** @var Post $model */")
        );
    }

    /// A re-opening `<?php` with no preceding `?>` is the file's very first
    /// tag, not a split block — there's nothing to skip past, and (since
    /// nothing precedes it) no docblock to find either.
    #[test]
    fn leading_reopen_tag_with_no_prior_close_finds_nothing() {
        let src = "<?php\n$x = 1;\n";
        let offset = src.find("$x").unwrap() as u32;
        assert_eq!(find_preceding_docblock(src, offset), None);
    }

    /// Guards against treating unrelated HTML/PHP alternation as a docblock
    /// boundary when there genuinely is no docblock to find.
    #[test]
    fn split_block_with_no_docblock_finds_nothing() {
        let src = "<?php\n$unrelated = 1;\n?>\n<div>\n<?php $x = 1;\n";
        let offset = src.rfind("$x").unwrap() as u32;
        assert_eq!(find_preceding_docblock(src, offset), None);
    }

    #[test]
    fn modifier_keywords_still_skip_after_a_split_block() {
        // Not a realistic PHP construct (a docblock + modifier can't
        // actually precede a re-opening tag), but exercises that the two
        // skip loops compose without infinite-looping or misordering.
        let src = "<?php\n/** doc */\nfinal\n?>\n<?php\n";
        let offset = src.len() as u32;
        assert_eq!(find_preceding_docblock(src, offset), Some("/** doc */"));
    }
}

#[cfg(test)]
mod skip_leading_attributes_tests {
    use super::skip_leading_attributes;

    fn after(src: &str) -> &str {
        &src[skip_leading_attributes(src, 0) as usize..]
    }

    #[test]
    fn skips_groups_comments_and_brackets_in_strings() {
        assert_eq!(after("#[A] #[B(['x' => \"]\"])]\nclass C {}"), "class C {}");
        assert_eq!(
            after("#[A, B('\\']')] /** doc */ // c\n# c\nfinal class C {}"),
            "final class C {}"
        );
        assert_eq!(after("public $x;"), "public $x;");
    }

    #[test]
    fn stops_at_end_of_unterminated_attribute() {
        assert_eq!(after("#[A('x"), "");
    }
}

#[cfg(test)]
mod attach_docblocks_tests {
    use php_ast::owned::{ClassMemberKind, EnumMemberKind, StmtKind};

    #[test]
    fn member_and_param_docblocks_after_attributes() {
        let src = "<?php\nclass C {\n    #[A]\n    /** p */\n    public $p;\n    public $plain;\n    #[A]\n    /** c */\n    const X = 1;\n    #[A] /** m */\n    public function m(#[A] /** q */ $q) {}\n}\nenum E {\n    #[A]\n    /** e */\n    case K;\n}\n";
        let mut parsed = php_rs_parser::parse(src);
        super::attach_docblocks_after_attributes(&mut parsed);
        let mut docs = Vec::new();
        for stmt in parsed.program.stmts.iter() {
            match &stmt.kind {
                StmtKind::Class(c) => {
                    for m in c.body.members.iter() {
                        match &m.kind {
                            ClassMemberKind::Property(p) => docs.push(p.doc_comment.clone()),
                            ClassMemberKind::ClassConst(k) => docs.push(k.doc_comment.clone()),
                            ClassMemberKind::Method(f) => {
                                docs.push(f.doc_comment.clone());
                                docs.push(f.params[0].doc_comment.clone());
                            }
                            _ => {}
                        }
                    }
                }
                StmtKind::Enum(e) => {
                    for m in e.body.members.iter() {
                        if let EnumMemberKind::Case(k) = &m.kind {
                            docs.push(k.doc_comment.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        let texts: Vec<_> = docs
            .iter()
            .map(|d| d.as_ref().map(|c| c.text.to_string()))
            .collect();
        let expected = [
            Some("/** p */"),
            None,
            Some("/** c */"),
            Some("/** m */"),
            Some("/** q */"),
            Some("/** e */"),
        ];
        assert_eq!(texts, expected.map(|t| t.map(str::to_string)));
    }
}
