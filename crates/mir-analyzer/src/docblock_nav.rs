//! Symbol lookup for names inside docblock type expressions.
//!
//! Operates on source text around the cursor (one line plus a bounded
//! backward scan), so a lookup never parses or analyzes the file.

use crate::db::{Fqcn, MirDatabase};
use crate::Name;

pub(crate) enum DocblockLookup {
    NotInDocblock,
    InDocblock(Option<Name>),
}

pub(crate) fn docblock_name_at(
    db: &dyn MirDatabase,
    file: &str,
    source: &str,
    offset: u32,
) -> DocblockLookup {
    let off = offset as usize;
    if off > source.len() || !source.is_char_boundary(off) {
        return DocblockLookup::NotInDocblock;
    }
    let line_start = source[..off].rfind('\n').map_or(0, |i| i + 1);
    let line_end = source[off..].find('\n').map_or(source.len(), |i| off + i);
    let line = &source[line_start..line_end];
    if !line.contains("/**") && !line.trim_start().starts_with('*') {
        return DocblockLookup::NotInDocblock;
    }
    if !inside_docblock(source, off) {
        return DocblockLookup::NotInDocblock;
    }
    DocblockLookup::InDocblock(
        type_word_at(line, off - line_start).and_then(|word| resolve(db, file, &word)),
    )
}

fn inside_docblock(source: &str, off: usize) -> bool {
    let Some(open) = source[..off].rfind("/*") else {
        return false;
    };
    source[open..].starts_with("/**") && !source[open + 2..off].contains("*/")
}

/// Tags whose first operand is a type expression.
fn is_type_tag(tag: &str) -> bool {
    let tag = ["psalm-", "phpstan-", "phan-"]
        .iter()
        .find_map(|p| tag.strip_prefix(p))
        .unwrap_or(tag);
    matches!(
        tag,
        "param"
            | "return"
            | "var"
            | "throws"
            | "see"
            | "property"
            | "property-read"
            | "property-write"
            | "method"
            | "extends"
            | "implements"
            | "use"
            | "mixin"
            | "template"
            | "template-covariant"
            | "template-contravariant"
            | "param-out"
            | "param-closure-this"
            | "assert"
            | "assert-if-true"
            | "assert-if-false"
            | "this-out"
            | "self-out"
            | "require-extends"
            | "require-implements"
    )
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '\\' || !c.is_ascii()
}

/// A name token at `cursor` (a byte index into `line`), qualified by its
/// `Class::` receiver when it is a member.
struct TypeWord {
    class: String,
    member: Option<Member>,
}

enum Member {
    Method(String),
    Property(String),
    Constant(String),
}

fn type_word_at(line: &str, cursor: usize) -> Option<TypeWord> {
    let at_tag = line[..cursor].rfind('@')?;
    let tag_len = line[at_tag + 1..]
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .unwrap_or(line.len() - at_tag - 1);
    let tag = &line[at_tag + 1..at_tag + 1 + tag_len];
    if !is_type_tag(tag) {
        return None;
    }
    let mut start = at_tag + 1 + tag_len;
    if tag.ends_with("template") || tag.contains("template-") {
        start = after_template_bound(line, start)?;
    }
    let start = start + (line[start..].len() - line[start..].trim_start().len());
    let end = type_expr_end(line, start);
    if cursor < start || cursor > end {
        return None;
    }
    word_with_member(&line[..end], start, cursor)
}

/// Offset just past `T of` / `T as` after a `@template` tag.
fn after_template_bound(line: &str, from: usize) -> Option<usize> {
    let rest = line[from..].trim_start();
    let name_end = rest.find(char::is_whitespace)?;
    let after_name = rest[name_end..].trim_start();
    let kw_end = after_name
        .find(char::is_whitespace)
        .unwrap_or(after_name.len());
    if !matches!(&after_name[..kw_end], "of" | "as") {
        return None;
    }
    Some(line.len() - after_name.len() + kw_end)
}

/// End of the type expression starting at `start`: the first whitespace at
/// bracket depth 0 that is not a continuation of a `|`, `&` or `,` operand.
fn type_expr_end(line: &str, start: usize) -> usize {
    let mut depth = 0i32;
    let mut prev = ' ';
    let bytes = line.as_bytes();
    for (i, c) in line[start..].char_indices() {
        let i = start + i;
        match c {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth -= 1,
            '$' if depth == 0 && !matches!(prev, ':') => return i,
            '*' if bytes.get(i + 1) == Some(&b'/') => return i,
            c if c.is_whitespace() && depth <= 0 => {
                let next = line[i..].trim_start().chars().next();
                let continues = matches!(prev, '|' | '&' | ',' | '<' | '(' | ':')
                    || matches!(next, Some('|' | '&'));
                if !continues {
                    return i;
                }
            }
            _ => {}
        }
        if !c.is_whitespace() {
            prev = c;
        }
    }
    line.len()
}

fn word_with_member(expr: &str, start: usize, cursor: usize) -> Option<TypeWord> {
    let mut ws = cursor;
    while ws > start {
        let prev = expr[..ws].chars().next_back()?;
        if !is_name_char(prev) {
            break;
        }
        ws -= prev.len_utf8();
    }
    let mut we = cursor;
    while let Some(c) = expr[we..].chars().next() {
        if !is_name_char(c) {
            break;
        }
        we += c.len_utf8();
    }
    let word = &expr[ws..we];
    if word.is_empty() || word.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let before = &expr[start..ws];
    if before.ends_with('$') && !before.ends_with("::$") {
        return None;
    }
    // Array-shape keys (`array{foo: int}`) are not types.
    let after = expr[we..].trim_start();
    if after.starts_with(':') && !after.starts_with("::") || after.starts_with("?:") {
        return None;
    }

    if let Some(recv_end) = before
        .strip_suffix("::")
        .or_else(|| before.strip_suffix("::$"))
    {
        let is_property = before.ends_with("::$");
        let class = receiver_word(recv_end)?;
        let member = if is_property {
            Member::Property(word.to_string())
        } else if expr[we..].starts_with('(') {
            Member::Method(word.to_string())
        } else {
            Member::Constant(word.to_string())
        };
        return Some(TypeWord {
            class,
            member: Some(member),
        });
    }
    Some(TypeWord {
        class: word.to_string(),
        member: None,
    })
}

fn receiver_word(text: &str) -> Option<String> {
    let start = text
        .char_indices()
        .rev()
        .take_while(|(_, c)| is_name_char(*c))
        .last()
        .map(|(i, _)| i)?;
    Some(text[start..].to_string())
}

fn resolve(db: &dyn MirDatabase, file: &str, word: &TypeWord) -> Option<Name> {
    if crate::diagnostics::is_docblock_keyword(&word.class)
        || matches!(
            word.class.to_ascii_lowercase().as_str(),
            "self" | "static" | "parent"
        )
    {
        return None;
    }
    let class = crate::db::resolve_docblock_type_name(db, file, &word.class);
    if !crate::db::class_exists(db, &class) {
        return None;
    }
    let Some(member) = &word.member else {
        return Some(Name::class(class));
    };
    let fqcn = Fqcn::from_str(db, &class);
    match member {
        Member::Method(name) => {
            let (owner, _) = crate::db::find_method_in_chain(db, fqcn, name)?;
            Some(Name::method(owner, name))
        }
        Member::Property(name) => {
            let (owner, _) = crate::db::find_property_in_chain(db, fqcn, name)?;
            Some(Name::property(owner, name.as_str()))
        }
        Member::Constant(name) => {
            let (owner, _) = crate::db::find_class_constant_in_chain(db, fqcn, name)?;
            Some(Name::class_constant(owner, name.as_str()))
        }
    }
}
