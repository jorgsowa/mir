/// Lowercase a PHP identifier (method name, function name, class name, keyword).
///
/// PHP identifiers are restricted to ASCII (letters, digits, underscores), so
/// `to_ascii_lowercase` is both correct and faster than Unicode-aware `to_lowercase`.
///
/// **Do not use** for docblock content, string literals, or arbitrary source text —
/// those may contain non-ASCII characters.
#[inline]
pub(crate) fn php_ident_lowercase(s: &str) -> String {
    s.to_ascii_lowercase()
}
