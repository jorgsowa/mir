//! Validation guards that prove a value's characters: `ctype_digit($x)`, or
//! `preg_match('/^[a-z0-9_]+$/', $x)`. A sink kind is safe once none of the
//! characters it needs to break out can appear.

use crate::ast_ext::ArgExt as _;
use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::FunctionRefExt as _;
use crate::ast_ext::NameBareExt as _;
use crate::ast_ext::NameResolveExt as _;
use mir_types::Name;
use php_ast::ast::{BinaryOp, CastKind};
use php_ast::owned::{Arg, Expr, ExprKind};

use crate::db::MirDatabase;
use crate::flow_state::FlowState;
use crate::taint::Taint;
use std::borrow::Cow;

/// A set of ASCII characters, plus whether any non-ASCII one may appear.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CharSet {
    ascii: u128,
    non_ascii: bool,
}

impl CharSet {
    fn of(chars: &str) -> CharSet {
        let mut set = CharSet::default();
        for c in chars.chars() {
            set.add(c);
        }
        set
    }

    fn range(from: char, to: char) -> CharSet {
        let mut set = CharSet::default();
        for c in from..=to {
            set.add(c);
        }
        set
    }

    fn add(&mut self, c: char) {
        if c.is_ascii() {
            self.ascii |= 1 << (c as u32);
        } else {
            self.non_ascii = true;
        }
    }

    fn union(self, other: CharSet) -> CharSet {
        CharSet {
            ascii: self.ascii | other.ascii,
            non_ascii: self.non_ascii || other.non_ascii,
        }
    }

    fn intersects(self, other: CharSet) -> bool {
        self.ascii & other.ascii != 0 || (self.non_ascii && other.non_ascii)
    }

    fn with_other_case(self) -> CharSet {
        let mut set = self;
        for c in 'a'..='z' {
            let upper = c.to_ascii_uppercase();
            if self.ascii & (1 << (c as u32)) != 0 || self.ascii & (1 << (upper as u32)) != 0 {
                set.add(c);
                set.add(upper);
            }
        }
        set
    }
}

fn digits() -> CharSet {
    CharSet::range('0', '9')
}

fn hex_digits() -> CharSet {
    digits()
        .union(CharSet::range('a', 'f'))
        .union(CharSet::range('A', 'F'))
}

fn letters() -> CharSet {
    CharSet::range('a', 'z').union(CharSet::range('A', 'Z'))
}

fn word() -> CharSet {
    letters().union(digits()).union(CharSet::of("_"))
}

fn space() -> CharSet {
    CharSet::of(" \t\n\r\x0b\x0c")
}

/// Kinds a value restricted to `allowed` can no longer break out of.
fn safe_kinds(allowed: CharSet) -> Taint {
    let rules: [(Taint, &str); 10] = [
        (Taint::HTML, "<>\"'&"),
        (Taint::SQL, "'\"\\`;()#/*= \t\n\r"),
        (Taint::SHELL, ";|&$`\\\"'<>(){}[]*?!~#% \t\n\r^"),
        (Taint::HEADER, "\r\n"),
        (Taint::COOKIE, "\r\n;, "),
        (Taint::FILE, "/\\.:"),
        (Taint::UNSERIALIZE, ":;{\""),
        (Taint::EVAL, "();$'\"`"),
        (Taint::LDAP, "*()\\&|!=<>~"),
        (Taint::XPATH, "'\"[]()/=@*|<>"),
    ];
    let mut safe = Taint::NONE;
    for (kind, dangerous) in rules {
        let mut dangerous = CharSet::of(dangerous);
        // A NUL byte breaks nothing out of markup, headers or cookies.
        if !(Taint::HTML | Taint::HEADER | Taint::COOKIE).intersects(kind) {
            dangerous.add('\0');
        }
        if !allowed.intersects(dangerous) {
            safe |= kind;
        }
    }
    // A function or class name needs a letter or an underscore.
    if !allowed.intersects(letters().union(CharSet::of("_\\"))) && !allowed.non_ascii {
        safe |= Taint::CALLABLE;
    }
    safe
}

/// Splits `/body/flags` into its body and modifiers.
fn split_pattern(pattern: &str) -> Option<(&str, &str)> {
    let delimiter = pattern.chars().next()?;
    if delimiter.is_alphanumeric() || delimiter == '\\' || delimiter.is_whitespace() {
        return None;
    }
    let closing = match delimiter {
        '(' => ')',
        '{' => '}',
        '[' => ']',
        '<' => '>',
        other => other,
    };
    let end = pattern.rfind(closing).filter(|&i| i > 0)?;
    Some((
        &pattern[delimiter.len_utf8()..end],
        &pattern[end + closing.len_utf8()..],
    ))
}

/// Characters every string accepted by the fully anchored `pattern` is made of.
fn pattern_chars(pattern: &str) -> Option<CharSet> {
    let (body, modifiers) = split_pattern(pattern)?;
    if modifiers.contains(['m', 'x', 's']) {
        return None;
    }
    let body = body
        .strip_prefix('^')
        .or_else(|| body.strip_prefix("\\A"))?;
    let (body, trailing_newline) = if let Some(b) = body.strip_suffix("\\z") {
        (b, false)
    } else if let Some(b) = body.strip_suffix("\\Z") {
        (b, true)
    } else {
        let b = body.strip_suffix('$')?;
        (b, !modifiers.contains('D'))
    };
    if b_ends_with_escape(body) {
        return None;
    }
    let mut set = body_chars(body)?;
    if trailing_newline {
        set.add('\n');
    }
    if modifiers.contains('i') {
        set = set.with_other_case();
    }
    Some(set)
}

/// Kinds that stay out of what `preg_replace($pattern, $replacement, $subject)` returns when the
/// pattern is one character class and the replacement a plain literal: whatever the class
/// matches is replaced, so only the other characters, or the replacement's own, can remain.
pub fn replace_safe_kinds(args: &[Arg]) -> Option<Taint> {
    if args.len() > 3 || args.iter().any(|a| a.name.is_some() || a.unpack()) {
        return None;
    }
    let replacement = string_arg(args, 1)?;
    // `$1` and `\1` copy the matched text back in.
    if replacement.contains(['$', '\\']) {
        return None;
    }
    let mut remaining = survivors(string_arg(args, 0)?)?;
    for c in replacement.chars() {
        remaining.add(c);
    }
    let safe = safe_kinds(remaining);
    (!safe.is_empty()).then_some(safe)
}

/// Characters a replaced single-class `pattern` leaves in the subject.
fn survivors(pattern: &str) -> Option<CharSet> {
    let (body, modifiers) = split_pattern(pattern)?;
    // `x` skips whitespace, `A` anchors the match, `e` evaluates the replacement.
    if modifiers.contains(|c| !"imsuUDSXJ".contains(c)) {
        return None;
    }
    let unicode = modifiers.contains('u');
    let mut chars = body.chars().peekable();
    let (matched, negated) = match chars.next()? {
        '[' => {
            let negated = chars.peek() == Some(&'^');
            if negated {
                chars.next();
            }
            (class_chars(&mut chars)?, negated)
        }
        '\\' => {
            let (class, negated) = match chars.next()? {
                'd' => (digits(), false),
                'D' => (digits(), true),
                'w' => (word(), false),
                'W' => (word(), true),
                's' => (space(), false),
                'S' => (space(), true),
                _ => return None,
            };
            (class, negated)
        }
        _ => return None,
    };
    // Each repetition is still a match of one character at a time.
    if !matches!(
        chars.collect::<String>().as_str(),
        "" | "+" | "*" | "?" | "+?" | "*?"
    ) {
        return None;
    }
    let matched = if modifiers.contains('i') {
        matched.with_other_case()
    } else {
        matched
    };
    let mut remaining = if negated {
        matched
    } else {
        CharSet {
            ascii: !matched.ascii,
            non_ascii: true,
        }
    };
    // Under `u` the shorthand classes also cover non-ASCII letters, digits and spaces.
    if unicode && body.contains('\\') {
        remaining.non_ascii = true;
    }
    Some(remaining)
}

fn b_ends_with_escape(body: &str) -> bool {
    body.chars().rev().take_while(|c| *c == '\\').count() % 2 == 1
}

fn escape_class(c: char) -> Option<CharSet> {
    Some(match c {
        'd' => digits(),
        'w' => word(),
        's' => space(),
        'D'
        | 'W'
        | 'S'
        | 'p'
        | 'P'
        | 'X'
        | 'R'
        | 'N'
        | 'C'
        | 'x'
        | 'u'
        | 'h'
        | 'v'
        | 'H'
        | 'V'
        | 'Q'
        | 'E'
        | 'K'
        | 'g'
        | 'k'
        | '0'..='9' => return None,
        't' => CharSet::of("\t"),
        'n' => CharSet::of("\n"),
        'r' => CharSet::of("\r"),
        'b' | 'B' | 'A' | 'z' | 'Z' | 'G' => CharSet::default(),
        other if other.is_ascii_punctuation() || other == ' ' => CharSet::of(&other.to_string()),
        _ => return None,
    })
}

/// Union of every character an anchored regex body can match; `None` for
/// constructs that match characters this can't bound (`.`, `[^…]`, `\D`, an
/// unanchored top-level `|`).
fn body_chars(body: &str) -> Option<CharSet> {
    let mut set = CharSet::default();
    let mut chars = body.chars().peekable();
    let mut depth = 0usize;
    while let Some(c) = chars.next() {
        match c {
            '\\' => set = set.union(escape_class(chars.next()?)?),
            '[' => set = set.union(class_chars(&mut chars)?),
            '.' => return None,
            '|' if depth == 0 => return None,
            '(' => {
                depth += 1;
                if chars.peek() == Some(&'?') {
                    chars.next();
                    match chars.next()? {
                        ':' | '=' | '!' | '>' => {}
                        '<' if matches!(chars.peek(), Some('=') | Some('!')) => {
                            chars.next();
                        }
                        _ => return None,
                    }
                }
            }
            ')' => depth = depth.checked_sub(1)?,
            '{' => {
                for q in chars.by_ref() {
                    if q == '}' {
                        break;
                    }
                    if !q.is_ascii_digit() && q != ',' {
                        return None;
                    }
                }
            }
            '*' | '+' | '?' | '|' => {}
            '^' | '$' => return None,
            literal => set.add(literal),
        }
    }
    (depth == 0).then_some(set)
}

fn class_chars(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<CharSet> {
    if chars.peek() == Some(&'^') {
        return None;
    }
    let mut set = CharSet::default();
    let mut previous: Option<char> = None;
    let mut first = true;
    loop {
        let c = chars.next()?;
        match c {
            ']' if !first => return Some(set),
            '\\' => {
                let escaped = chars.next()?;
                let class = escape_class(escaped)?;
                set = set.union(class);
                previous = (class.ascii.count_ones() == 1 && !class.non_ascii)
                    .then(|| char::from_u32(class.ascii.trailing_zeros()))
                    .flatten();
            }
            '-' if previous.is_some() && chars.peek().is_some_and(|n| *n != ']') => {
                let from = previous.take()?;
                let to = match chars.next()? {
                    '\\' => chars.next()?,
                    '[' => return None,
                    to => to,
                };
                if from > to {
                    return None;
                }
                set = set.union(CharSet::range(from, to));
            }
            '[' if chars.peek() == Some(&':') => return None,
            literal => {
                set.add(literal);
                previous = Some(literal);
            }
        }
        first = false;
    }
}

fn string_arg(args: &[Arg], index: usize) -> Option<&str> {
    match &args.get(index)?.value()?.kind {
        ExprKind::String(s) => Some(s.as_str()),
        _ => None,
    }
}

/// A `preg_match` pattern: a literal, or a class or global constant holding one.
fn pattern_arg<'e>(args: &'e [Arg], scope: GuardScope<'_>) -> Option<Cow<'e, str>> {
    if let Some(literal) = string_arg(args, 0) {
        return Some(Cow::Borrowed(literal));
    }
    let pattern = args.first()?.value()?;
    if !matches!(
        pattern.kind,
        ExprKind::Name(_) | ExprKind::ClassConstAccess(_)
    ) {
        return None;
    }
    let ty = crate::expr::constant_expr_type(pattern, scope.ctx, scope.db, scope.file)?;
    match ty.types.as_slice() {
        [mir_types::Atomic::TLiteralString(s)] => Some(Cow::Owned(s.to_string())),
        _ => None,
    }
}

/// What a passing guard call validated.
#[derive(Debug, PartialEq, Eq)]
pub enum Guarded<'a> {
    Var(&'a str),
    Element {
        var: &'a str,
        key: mir_types::Name,
    },
    /// Under the CLI, `$_SERVER` holds the process environment, not client data.
    CliServerArray,
}

/// The value a passing guard call validates, and the kinds it is then safe for.
pub fn guarded_var<'e>(call: &'e Expr, scope: GuardScope<'_>) -> Option<(Guarded<'e>, Taint)> {
    guard_of_call(call, scope).map(|(guarded, safe, _)| (guarded, safe))
}

/// Where a guard's constants and maps get their types.
#[derive(Clone, Copy)]
pub struct GuardScope<'a> {
    pub ctx: &'a FlowState,
    pub db: &'a dyn MirDatabase,
    pub file: &'a str,
}

/// What `cond` proves about a guarded value once it evaluates to `holds`: the bare call, or the
/// call compared with the result it returns on a match (`preg_match(..) === 1`, `ctype_digit(..) !== false`).
pub fn guarded_when<'e>(
    cond: &'e Expr,
    holds: bool,
    scope: GuardScope<'_>,
) -> Option<(Guarded<'e>, Taint)> {
    let ExprKind::Binary(b) = &cond.kind else {
        if !holds {
            return guarded_by_failed_match(cond, scope);
        }
        return guarded_var(cond, scope)
            .or_else(|| cli_sapi_list(cond))
            .or_else(|| guarded_by_list(cond, scope))
            .or_else(|| guarded_by_map_key(cond, scope));
    };
    if let Some(guard) = guarded_by_cli_sapi(b, holds) {
        return Some(guard);
    }
    if let Some(guard) = guarded_by_literal(b, holds, scope) {
        return Some(guard);
    }
    let (call, literal, op) = match (&b.left.kind, &b.right.kind) {
        (ExprKind::Int(_) | ExprKind::Bool(_), _) => (&b.right, &b.left.kind, flipped(b.op)?),
        (_, ExprKind::Int(_) | ExprKind::Bool(_)) => (&b.left, &b.right.kind, b.op),
        _ => return None,
    };
    let (guarded, safe, is_preg) = guard_of_call(call, scope)?;
    let matched_when = match *literal {
        ExprKind::Int(n) => matched_when_int(op, n)?,
        // `preg_match()` never returns a bool on a match.
        ExprKind::Bool(v) if !is_preg => matched_when_bool(op, v)?,
        _ => return None,
    };
    (holds == matched_when).then_some((guarded, safe))
}

fn is_sapi_name(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::Name(_) => expr.kind.name_bare() == Some("PHP_SAPI"),
        ExprKind::FunctionCall(fc) => {
            fc.args.is_empty()
                && fc
                    .name
                    .kind
                    .name_bare()
                    .is_some_and(|n| n.eq_ignore_ascii_case("php_sapi_name"))
        }
        _ => false,
    }
}

fn is_cli_sapi_literal(expr: &Expr) -> bool {
    matches!(&expr.kind, ExprKind::String(s) if matches!(s.as_str(), "cli" | "phpdbg"))
}

/// `PHP_SAPI === 'cli'`, `php_sapi_name() !== 'cli'` and their loose forms.
fn guarded_by_cli_sapi(
    b: &php_ast::owned::BinaryExpr,
    holds: bool,
) -> Option<(Guarded<'_>, Taint)> {
    let matched_when = match b.op {
        BinaryOp::Identical | BinaryOp::Equal => true,
        BinaryOp::NotIdentical | BinaryOp::NotEqual => false,
        _ => return None,
    };
    let names_cli = (is_sapi_name(&b.left) && is_cli_sapi_literal(&b.right))
        || (is_sapi_name(&b.right) && is_cli_sapi_literal(&b.left));
    (names_cli && holds == matched_when).then_some((Guarded::CliServerArray, Taint::ALL))
}

/// `in_array(PHP_SAPI, ['cli', 'phpdbg'], true)`.
fn cli_sapi_list(call: &Expr) -> Option<(Guarded<'_>, Taint)> {
    let ExprKind::FunctionCall(fc) = &call.kind else {
        return None;
    };
    if !fc.name.kind.name_bare()?.eq_ignore_ascii_case("in_array")
        || !is_sapi_name(fc.args.first()?.value()?)
    {
        return None;
    }
    let ExprKind::Array(items) = &fc.args.get(1)?.value()?.kind else {
        return None;
    };
    (!items.is_empty() && items.iter().all(|item| is_cli_sapi_literal(item.value())))
        .then_some((Guarded::CliServerArray, Taint::ALL))
}

/// A failed `preg_match('/[<>]/', $x)` proves none of the class's characters occur in `$x`.
fn guarded_by_failed_match<'e>(
    call: &'e Expr,
    scope: GuardScope<'_>,
) -> Option<(Guarded<'e>, Taint)> {
    let ExprKind::FunctionCall(fc) = &call.kind else {
        return None;
    };
    if !fc.name.kind.name_bare()?.eq_ignore_ascii_case("preg_match") {
        return None;
    }
    let value = fc.args.get(1)?;
    if value.name.is_some() || value.unpack() {
        return None;
    }
    let pattern = pattern_arg(&fc.args, scope)?;
    let pattern = pattern.as_ref();
    // Invalid UTF-8 makes a `/u` match return false whatever the value holds.
    if split_pattern(pattern)?.1.contains('u') {
        return None;
    }
    let safe = safe_kinds(survivors(pattern)?);
    (!safe.is_empty()).then_some((guarded_subject(value.value()?)?, safe))
}

/// `$x === 'lit'` proves the value is the program's own literal; `$x == 'lit'` as far as
/// [`case_safe_kinds`] allows.
fn guarded_by_literal<'e>(
    b: &'e php_ast::owned::BinaryExpr,
    holds: bool,
    scope: GuardScope<'_>,
) -> Option<(Guarded<'e>, Taint)> {
    let is_scalar_constant = |e: &Expr| {
        crate::expr::constant_expr_type(e, scope.ctx, scope.db, scope.file).is_some_and(|ty| {
            matches!(
                ty.types.as_slice(),
                [mir_types::Atomic::TLiteralString(_) | mir_types::Atomic::TLiteralInt(_)]
            )
        })
    };
    let (subject, literal) = if is_literal(&b.right) || is_scalar_constant(&b.right) {
        (&b.left, &b.right)
    } else if is_literal(&b.left) || is_scalar_constant(&b.left) {
        (&b.right, &b.left)
    } else {
        return None;
    };
    let (matched_when, comparison_safe) = match b.op {
        BinaryOp::Identical => (true, Taint::ALL),
        BinaryOp::NotIdentical => (false, Taint::ALL),
        BinaryOp::Equal => (true, case_safe_kinds(literal)?),
        BinaryOp::NotEqual => (false, case_safe_kinds(literal)?),
        _ => return None,
    };
    if holds != matched_when {
        return None;
    }
    let (guarded, safe) = compared_subject(subject)?;
    Some((guarded, safe.intersection(comparison_safe)))
}

/// `in_array($x, ['a', 'b'], true)` proves the value is one of the listed literals.
fn guarded_by_list<'e>(call: &'e Expr, scope: GuardScope<'_>) -> Option<(Guarded<'e>, Taint)> {
    let ExprKind::FunctionCall(fc) = &call.kind else {
        return None;
    };
    let name = fc.name.kind.name_bare()?;
    if !crate::util::php_ident_lowercase(name).eq("in_array")
        || fc.args.iter().any(|a| a.name.is_some() || a.unpack())
    {
        return None;
    }
    let strict = matches!(
        fc.args.get(2).and_then(|a| a.value()).map(|v| &v.kind),
        Some(ExprKind::Bool(true))
    );
    let haystack = fc.args.get(1)?.value()?;
    let mut comparison_safe = Taint::ALL;
    if let ExprKind::Array(items) = &haystack.kind {
        for item in items.iter() {
            if item.is_spread() || item.is_by_ref() || !is_literal(item.value()) {
                return None;
            }
            if !strict {
                comparison_safe = comparison_safe.intersection(case_safe_kinds(item.value())?);
            }
        }
    } else if !strict
        || crate::narrowing::extract_haystack_type(haystack, scope.ctx, scope.db, scope.file)
            .is_none()
    {
        return None;
    }
    let (guarded, safe) = compared_subject(fc.args.first()?.value()?)?;
    Some((guarded, safe.intersection(comparison_safe)))
}

/// The value a comparison checks, and the kinds an exact match leaves it safe for: changing the
/// case adds no metacharacter, while trimming hides whitespace a line break can be part of.
fn compared_subject(expr: &Expr) -> Option<(Guarded<'_>, Taint)> {
    if let ExprKind::FunctionCall(fc) = &expr.kind {
        let [arg] = &fc.args[..] else {
            return None;
        };
        if arg.name.is_some() || arg.unpack() {
            return None;
        }
        let name = fc.name.kind.name_bare()?;
        let safe = match crate::util::php_ident_lowercase(name).as_str() {
            "strtolower" | "strtoupper" | "mb_strtolower" | "mb_strtoupper" => Taint::ALL,
            "trim" | "ltrim" | "rtrim" => Taint::ALL.without(Taint::HEADER | Taint::COOKIE),
            _ => return None,
        };
        return Some((guarded_subject(arg.value()?)?, safe));
    }
    Some((guarded_subject(expr)?, Taint::ALL))
}

/// `isset($map[$x])` or `array_key_exists($x, $map)` proves the value is one of the map's keys.
fn guarded_by_map_key<'e>(cond: &'e Expr, scope: GuardScope<'_>) -> Option<(Guarded<'e>, Taint)> {
    let (key, map) = match &cond.kind {
        ExprKind::Isset(vars) => {
            let [var] = &vars[..] else {
                return None;
            };
            let ExprKind::ArrayAccess(aa) = &var.kind else {
                return None;
            };
            (aa.index.as_ref()?, &aa.array)
        }
        ExprKind::FunctionCall(fc) => {
            let name = fc.name.kind.name_bare()?;
            if !crate::util::php_ident_lowercase(name).eq("array_key_exists")
                || fc.args.iter().any(|a| a.name.is_some() || a.unpack())
            {
                return None;
            }
            (fc.args.first()?.value()?, fc.args.get(1)?.value()?)
        }
        _ => return None,
    };
    let map_type = match &map.kind {
        ExprKind::Variable(var) => scope.ctx.get_var(var),
        _ => crate::expr::constant_expr_type(map, scope.ctx, scope.db, scope.file)?,
    };
    let only_listed_keys = !map_type.types.is_empty()
        && map_type
            .types
            .iter()
            .all(|atom| matches!(atom, mir_types::Atomic::TKeyedArray { is_open: false, .. }));
    only_listed_keys.then(|| compared_subject(key)).flatten()
}

fn is_literal(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::String(_)
            | ExprKind::Int(_)
            | ExprKind::Float(_)
            | ExprKind::Bool(_)
            | ExprKind::Null
    )
}

/// Kinds a switch subject is safe for once `==` matched `literal`: a number literal also admits
/// whitespace-padded numeric strings, and `"1e3" == "1000"` compares numerically.
fn case_safe_kinds(literal: &Expr) -> Option<Taint> {
    let numeric = || safe_kinds(digits().union(CharSet::of(" \t\n\r\x0b\x0c+-.eE")));
    match &literal.kind {
        ExprKind::String(s) if !looks_numeric(s.as_str()) => Some(Taint::ALL),
        ExprKind::String(_) | ExprKind::Int(_) | ExprKind::Float(_) => Some(numeric()),
        _ => None,
    }
}

fn looks_numeric(text: &str) -> bool {
    let rest = text.trim_start_matches([' ', '\t', '\n', '\r', '\x0b', '\x0c']);
    let rest = rest.strip_prefix(['+', '-']).unwrap_or(rest);
    let rest = rest.strip_prefix('.').unwrap_or(rest);
    rest.starts_with(|c: char| c.is_ascii_digit())
}

/// Kinds a switch subject is safe for inside a body reached only through `case` labels spelling `labels`.
pub fn switch_guard<'a>(subject: &'a Expr, labels: &[&Expr]) -> Option<(Guarded<'a>, Taint)> {
    let guarded = guarded_subject(subject)?;
    let mut safe = Taint::ALL;
    for label in labels {
        safe = safe.intersection(case_safe_kinds(label)?);
    }
    (!safe.is_empty()).then_some((guarded, safe))
}

fn flipped(op: BinaryOp) -> Option<BinaryOp> {
    Some(match op {
        BinaryOp::Identical | BinaryOp::NotIdentical | BinaryOp::Equal | BinaryOp::NotEqual => op,
        BinaryOp::Less => BinaryOp::Greater,
        BinaryOp::Greater => BinaryOp::Less,
        BinaryOp::LessOrEqual => BinaryOp::GreaterOrEqual,
        BinaryOp::GreaterOrEqual => BinaryOp::LessOrEqual,
        _ => return None,
    })
}

/// The truthiness of `call <op> n` under which `preg_match()` returned 1 (not 0 or false).
fn matched_when_int(op: BinaryOp, n: i64) -> Option<bool> {
    match (op, n) {
        (BinaryOp::Identical | BinaryOp::Equal, 1)
        | (BinaryOp::Greater, 0)
        | (BinaryOp::GreaterOrEqual, 1)
        | (BinaryOp::NotEqual, 0) => Some(true),
        (BinaryOp::NotIdentical | BinaryOp::NotEqual, 1)
        | (BinaryOp::LessOrEqual, 0)
        | (BinaryOp::Less, 1)
        | (BinaryOp::Equal, 0) => Some(false),
        _ => None,
    }
}

fn matched_when_bool(op: BinaryOp, value: bool) -> Option<bool> {
    match op {
        BinaryOp::Identical | BinaryOp::Equal => Some(value),
        BinaryOp::NotIdentical | BinaryOp::NotEqual => Some(!value),
        _ => None,
    }
}

fn guard_of_call<'e>(call: &'e Expr, scope: GuardScope<'_>) -> Option<(Guarded<'e>, Taint, bool)> {
    let fc = match &call.kind {
        ExprKind::FunctionCall(fc) => fc,
        ExprKind::StaticMethodCall(smc) => return static_helper_guard(smc, scope),
        _ => return None,
    };
    let name = fc.name.kind.name_bare()?;
    let lower = crate::util::php_ident_lowercase(name);
    if lower == "hash_equals" {
        let payload = mac_checked_payload(&fc.args, scope)?;
        return Some((guarded_subject(payload)?, Taint::UNSERIALIZE, false));
    }
    let (safe, subject) = match lower.as_str() {
        "preg_match" => (
            safe_kinds(pattern_chars(&pattern_arg(&fc.args, scope)?)?),
            1,
        ),
        "filter_var" => (validating_filter_safe_kinds(&fc.args)?, 0),
        "str_starts_with" => (fixed_origin_prefix_safe_kinds(&fc.args, scope)?, 0),
        _ => match value_check_safe_kinds(&lower) {
            Some(safe) => (safe, 0),
            None => {
                let function = fc.name.kind.function_ref(scope.db, scope.file)?;
                let (fqn, f) = function.find(scope.db, scope.file)?;
                let (index, safe) =
                    helper_guard(f.location.as_ref()?, (Name::new(fqn), None), scope)?;
                (safe, index)
            }
        },
    };
    let guarded = guarded_subject(positional_arg(&fc.args, subject)?)?;
    (!safe.is_empty()).then_some((guarded, safe, lower == "preg_match"))
}

/// A call to `Class::helper($x)` whose body returns a guard on that parameter.
fn static_helper_guard<'e>(
    smc: &'e php_ast::owned::StaticMethodCallExpr,
    scope: GuardScope<'_>,
) -> Option<(Guarded<'e>, Taint, bool)> {
    let class = match smc
        .class
        .kind
        .resolve_name_ref(scope.db, scope.file)?
        .as_str()
    {
        "self" | "static" => scope.ctx.self_fqcn.as_deref()?.to_string(),
        "parent" => scope.ctx.parent_fqcn.as_deref()?.to_string(),
        name => name.to_string(),
    };
    let ExprKind::Identifier(method) = &smc.method.kind else {
        return None;
    };
    let method = crate::util::php_ident_lowercase(method);
    let (owner, m) = crate::db::find_method_respecting_precedence(
        scope.db,
        crate::db::Fqcn::from_str(scope.db, &class),
        &method,
    )?;
    let key = (Name::new(&owner), Some(Name::new(&method)));
    let (index, safe) = helper_guard(m.location.as_ref()?, key, scope)?;
    Some((
        guarded_subject(positional_arg(&smc.args, index)?)?,
        safe,
        false,
    ))
}

/// The parameter a user helper's body guards, and the kinds a passing call leaves it safe for.
fn helper_guard(
    location: &mir_types::Location,
    key: (Name, Option<Name>),
    scope: GuardScope<'_>,
) -> Option<(usize, Taint)> {
    if crate::stubs::is_stub_path(location.file.as_ref()) {
        return None;
    }
    let file = scope.db.lookup_source_file(location.file.as_ref())?;
    crate::db::taint_sources::file_guard_helpers(scope.db, file)
        .get(&key)
        .copied()
}

/// The argument bound to parameter `index` when no earlier argument is named or spread.
fn positional_arg(args: &[Arg], index: usize) -> Option<&Expr> {
    let arg = args.get(index)?;
    (!args[..=index]
        .iter()
        .any(|a| a.name.is_some() || a.unpack()))
    .then(|| arg.value())
    .flatten()
}

/// The parameter a body consisting of `return <guard>;` checks, and the kinds it proves safe.
pub(crate) fn returned_param_guard(
    params: &[php_ast::owned::Param],
    stmts: &[php_ast::owned::Stmt],
    scope: GuardScope<'_>,
) -> Option<(usize, Taint)> {
    let [stmt] = stmts else {
        return None;
    };
    let php_ast::owned::StmtKind::Return(Some(cond)) = &stmt.kind else {
        return None;
    };
    let (Guarded::Var(var), safe) = guarded_when(cond, true, scope)? else {
        return None;
    };
    let index = params
        .iter()
        .position(|p| p.name.as_str() == Some(var.trim_start_matches('$')))?;
    let param = &params[index];
    (!param.by_ref && !param.variadic).then_some((index, safe))
}

/// The data of a `hash_hmac` side of `hash_equals`: matching proves whoever holds the key made it.
fn mac_checked_payload<'e>(args: &'e [Arg], scope: GuardScope<'_>) -> Option<&'e Expr> {
    let [known, user] = args else {
        return None;
    };
    [known, user].into_iter().find_map(|arg| {
        let ExprKind::FunctionCall(fc) = &arg.value()?.kind else {
            return None;
        };
        let [_, data, key, ..] = &fc.args[..] else {
            return None;
        };
        let untainted_key = key.name.is_none()
            && crate::taint::expr_taint(key.value()?, scope.ctx, scope.db, scope.file).is_empty();
        (fc.name.kind.name_bare()?.eq_ignore_ascii_case("hash_hmac")
            && data.name.is_none()
            && untainted_key)
            .then(|| data.value())
            .flatten()
    })
}

/// `str_starts_with($u, 'https://host/')` fixes the scheme and host, as a literal prefix does.
fn fixed_origin_prefix_safe_kinds(args: &[Arg], scope: GuardScope<'_>) -> Option<Taint> {
    let [_, prefix] = args else {
        return None;
    };
    (prefix.name.is_none()
        && crate::taint::starts_with_fixed_url_origin(
            prefix.value()?,
            scope.ctx,
            scope.db,
            scope.file,
        ))
    .then_some(Taint::SSRF | Taint::FILE)
}

/// Kinds a value is safe for once the one-argument check `name_lower` accepted it.
pub fn value_check_safe_kinds(name_lower: &str) -> Option<Taint> {
    Some(match name_lower {
        "ctype_digit" => safe_kinds(digits()),
        "ctype_xdigit" => safe_kinds(hex_digits()),
        "ctype_alpha" => safe_kinds(letters()),
        "ctype_alnum" => safe_kinds(letters().union(digits())),
        "ctype_lower" => safe_kinds(CharSet::range('a', 'z')),
        "ctype_upper" => safe_kinds(CharSet::range('A', 'Z')),
        "is_numeric" => padded_safe_kinds(digits().union(CharSet::of("+-.eE"))),
        "is_int" | "is_integer" | "is_long" | "is_float" | "is_double" | "is_bool" => Taint::ALL,
        _ => return None,
    })
}

/// Kinds a value made of `chars`, possibly padded with whitespace, is safe for: only a line break
/// in the padding can still do harm.
fn padded_safe_kinds(chars: CharSet) -> Taint {
    safe_kinds(chars).without(Taint::HEADER | Taint::COOKIE)
}

/// Kinds a value is safe for once `filter_var($x, FILTER_VALIDATE_*)` accepted it. Options can
/// supply a default or a null failure value, so only the two-argument form proves anything.
fn validating_filter_safe_kinds(args: &[Arg]) -> Option<Taint> {
    if args.len() != 2 {
        return None;
    }
    let filter = args[1].value()?.kind.name_bare()?.to_ascii_uppercase();
    Some(match filter.as_str() {
        "FILTER_VALIDATE_INT" => padded_safe_kinds(digits().union(CharSet::of("+-"))),
        "FILTER_VALIDATE_FLOAT" => padded_safe_kinds(digits().union(CharSet::of("+-.eE"))),
        "FILTER_VALIDATE_BOOL" | "FILTER_VALIDATE_BOOLEAN" => {
            padded_safe_kinds(letters().union(digits()))
        }
        "FILTER_VALIDATE_IP" => safe_kinds(hex_digits().union(CharSet::of(":."))),
        "FILTER_VALIDATE_MAC" => safe_kinds(hex_digits().union(CharSet::of(":-."))),
        _ => return None,
    })
}

/// A passing `preg_match($pattern, $x, $m)` bounds `$m` as it bounds `$x`: captures are substrings.
pub fn guarded_captures<'e>(
    cond: &'e Expr,
    holds: bool,
    scope: GuardScope<'_>,
) -> Option<(Guarded<'e>, Taint)> {
    let (_, safe) = guarded_when(cond, holds, scope)?;
    Some((Guarded::Var(match_captures(cond)?), safe))
}

/// The `$m` of `preg_match($pattern, $x, $m)`, alone or in a comparison.
pub fn match_captures(cond: &Expr) -> Option<&str> {
    let call = match &cond.kind {
        ExprKind::Binary(b) if matches!(b.left.kind, ExprKind::FunctionCall(_)) => &b.left,
        ExprKind::Binary(b) => &b.right,
        _ => cond,
    };
    let ExprKind::FunctionCall(fc) = &call.kind else {
        return None;
    };
    if !fc.name.kind.name_bare()?.eq_ignore_ascii_case("preg_match") {
        return None;
    }
    match &fc.args.get(2)?.value()?.kind {
        ExprKind::Variable(var) => Some(var),
        _ => None,
    }
}

/// `$x`, `$a['k']`, or either under `?? default`: when the left side is unset there is nothing to taint.
fn guarded_subject(expr: &Expr) -> Option<Guarded<'_>> {
    match &expr.kind {
        ExprKind::Variable(var) => Some(Guarded::Var(var)),
        ExprKind::Parenthesized(inner) | ExprKind::Cast(CastKind::String, inner) => {
            guarded_subject(inner)
        }
        ExprKind::NullCoalesce(nc) => guarded_subject(&nc.left),
        ExprKind::ArrayAccess(aa) => {
            let ExprKind::Variable(var) = &aa.array.kind else {
                return None;
            };
            let key = crate::expr::assignment::literal_key_name(aa.index.as_ref())?;
            Some(Guarded::Element {
                var: var.trim_start_matches('$'),
                key,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchored_patterns_bound_their_characters() {
        let expected = CharSet::range('a', 'z')
            .union(digits())
            .union(CharSet::of("_-"));
        assert_eq!(pattern_chars("/^[a-z0-9_-]+$/D"), Some(expected));
        assert!(pattern_chars("/[a-z]+/").is_none());
        assert!(pattern_chars("/^.+$/").is_none());
        assert!(pattern_chars("/^[^<]+$/").is_none());
        assert!(pattern_chars("/^a|b$/").is_none());
        assert!(pattern_chars("/^(?:yes|no)$/").is_some());
        assert!(pattern_chars("/^\\d{1,5}$/m").is_none());
    }

    #[test]
    fn replaced_class_leaves_the_other_characters() {
        let kept = survivors("/[^0-9]/").unwrap();
        assert_eq!(kept.ascii, digits().ascii);
        assert!(!kept.non_ascii);
        assert_eq!(survivors("/\\D+/").unwrap().ascii, digits().ascii);
        assert_eq!(survivors("/[^a-z]/i").unwrap().ascii, letters().ascii);
        let removed = survivors("/[<>]/").unwrap();
        assert!(!CharSet::of("<>").intersects(CharSet {
            ascii: removed.ascii,
            non_ascii: false
        }));
        assert!(removed.non_ascii);
        assert!(survivors("/[^0-9]{2}/").is_none());
        assert!(survivors("/[^0-9]|x/").is_none());
        assert!(survivors("/ [^0-9]/x").is_none());
        assert!(survivors("/[^0-9]/e").is_none());
        assert!(survivors("/^[^0-9]/").is_none());
    }

    #[test]
    fn dollar_allows_a_trailing_newline() {
        let set = pattern_chars("/^[a-z]+$/").unwrap();
        assert!(set.intersects(CharSet::of("\n")));
        assert!(!safe_kinds(set).intersects(Taint::HEADER));
        assert!(safe_kinds(set).intersects(Taint::HTML));
        let strict = pattern_chars("/^[a-z]+\\z/").unwrap();
        assert!(safe_kinds(strict).intersects(Taint::HEADER));
    }

    #[test]
    fn letters_still_name_callables() {
        let kinds = safe_kinds(letters());
        assert!(kinds.intersects(Taint::SQL));
        assert!(!kinds.intersects(Taint::CALLABLE));
        assert!(safe_kinds(digits()).intersects(Taint::CALLABLE));
    }

    #[test]
    fn numeric_case_labels_keep_whitespace_kinds_tainted() {
        assert!(looks_numeric(" -1.5e3"));
        assert!(looks_numeric(".5"));
        assert!(!looks_numeric("x1"));
        assert!(!looks_numeric(""));
        let int = |n| php_ast::owned::Expr {
            kind: ExprKind::Int(n),
            span: Default::default(),
        };
        let kinds = case_safe_kinds(&int(1)).unwrap();
        assert!(kinds.intersects(Taint::HTML));
        assert!(!kinds.intersects(Taint::HEADER));
    }
}
