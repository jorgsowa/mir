//! Validation guards that prove a value's characters: `ctype_digit($x)`, or
//! `preg_match('/^[a-z0-9_]+$/', $x)`. A sink kind is safe once none of the
//! characters it needs to break out can appear.

use crate::ast_ext::ByteStringExt as _;
use crate::ast_ext::NameRefExt as _;
use crate::ast_ext::{ArgExt as _, ArrayElementExt as _};
use php_ast::ast::BinaryOp;
use php_ast::owned::{Arg, Expr, ExprKind};

use crate::taint::Taint;

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

/// What a passing guard call validated.
#[derive(Debug, PartialEq, Eq)]
pub enum Guarded<'a> {
    Var(&'a str),
    Element { var: &'a str, key: mir_types::Name },
}

/// The value a passing guard call validates, and the kinds it is then safe for.
pub fn guarded_var(call: &Expr) -> Option<(Guarded<'_>, Taint)> {
    guard_of_call(call).map(|(guarded, safe, _)| (guarded, safe))
}

/// What `cond` proves about a guarded value once it evaluates to `holds`: the bare call, or the
/// call compared with the result it returns on a match (`preg_match(..) === 1`, `ctype_digit(..) !== false`).
pub fn guarded_when(cond: &Expr, holds: bool) -> Option<(Guarded<'_>, Taint)> {
    let ExprKind::Binary(b) = &cond.kind else {
        return holds
            .then(|| guarded_var(cond).or_else(|| guarded_by_list(cond)))
            .flatten();
    };
    if let Some(guard) = guarded_by_literal(b, holds) {
        return Some(guard);
    }
    let (call, literal, op) = match (&b.left.kind, &b.right.kind) {
        (ExprKind::Int(_) | ExprKind::Bool(_), _) => (&*b.right, &b.left.kind, flipped(b.op)?),
        (_, ExprKind::Int(_) | ExprKind::Bool(_)) => (&*b.left, &b.right.kind, b.op),
        _ => return None,
    };
    let (guarded, safe, is_preg) = guard_of_call(call)?;
    let matched_when = match *literal {
        ExprKind::Int(n) => matched_when_int(op, n)?,
        // `preg_match()` never returns a bool on a match.
        ExprKind::Bool(v) if !is_preg => matched_when_bool(op, v)?,
        _ => return None,
    };
    (holds == matched_when).then_some((guarded, safe))
}

/// `$x === 'lit'` proves the value is the program's own literal; the loose `==` also admits
/// other values, which a variable's type narrowing doesn't treat as a proof either.
fn guarded_by_literal(b: &php_ast::owned::BinaryExpr, holds: bool) -> Option<(Guarded<'_>, Taint)> {
    let subject = if is_literal(&b.right) {
        &*b.left
    } else if is_literal(&b.left) {
        &*b.right
    } else {
        return None;
    };
    let identical = match b.op {
        BinaryOp::Identical => true,
        BinaryOp::NotIdentical => false,
        _ => return None,
    };
    (holds == identical).then_some((guarded_subject(subject)?, Taint::ALL))
}

/// `in_array($x, ['a', 'b'], true)` proves the value is one of the listed literals.
fn guarded_by_list(call: &Expr) -> Option<(Guarded<'_>, Taint)> {
    let ExprKind::FunctionCall(fc) = &call.kind else {
        return None;
    };
    let name = fc.name.kind.name_ref()?;
    if !crate::util::php_ident_lowercase(name.trim_start_matches('\\')).eq("in_array")
        || fc.args.iter().any(|a| a.name.is_some() || a.unpack())
    {
        return None;
    }
    let strict = matches!(
        fc.args.get(2).and_then(|a| a.value()).map(|v| &v.kind),
        Some(ExprKind::Bool(true))
    );
    let ExprKind::Array(items) = &fc.args.get(1)?.value()?.kind else {
        return None;
    };
    if !strict
        || items
            .iter()
            .any(|item| item.unpack() || item.by_ref() || !is_literal(item.value()))
    {
        return None;
    }
    Some((guarded_subject(fc.args.first()?.value()?)?, Taint::ALL))
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

fn guard_of_call(call: &Expr) -> Option<(Guarded<'_>, Taint, bool)> {
    let ExprKind::FunctionCall(fc) = &call.kind else {
        return None;
    };
    let name = fc.name.kind.name_ref()?;
    let lower = crate::util::php_ident_lowercase(name.trim_start_matches('\\'));
    let (allowed, subject) = match lower.as_str() {
        "ctype_digit" => (digits(), 0),
        "ctype_xdigit" => (
            digits()
                .union(CharSet::range('a', 'f'))
                .union(CharSet::range('A', 'F')),
            0,
        ),
        "ctype_alpha" => (letters(), 0),
        "ctype_alnum" => (letters().union(digits()), 0),
        "ctype_lower" => (CharSet::range('a', 'z'), 0),
        "ctype_upper" => (CharSet::range('A', 'Z'), 0),
        "preg_match" => (pattern_chars(string_arg(&fc.args, 0)?)?, 1),
        _ => return None,
    };
    let value = fc.args.get(subject)?;
    if value.name.is_some() || value.unpack() {
        return None;
    }
    let guarded = guarded_subject(value.value()?)?;
    let safe = safe_kinds(allowed);
    (!safe.is_empty()).then_some((guarded, safe, lower == "preg_match"))
}

/// `$x`, `$a['k']`, or either under `?? default`: when the left side is unset there is nothing to taint.
fn guarded_subject(expr: &Expr) -> Option<Guarded<'_>> {
    match &expr.kind {
        ExprKind::Variable(var) => Some(Guarded::Var(var.trim_start_matches('$'))),
        ExprKind::Parenthesized(inner) => guarded_subject(inner),
        ExprKind::NullCoalesce(nc) => guarded_subject(&nc.left),
        ExprKind::ArrayAccess(aa) => {
            let ExprKind::Variable(var) = &aa.array.kind else {
                return None;
            };
            let key = crate::expr::assignment::literal_key_name(aa.index.as_deref())?;
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
