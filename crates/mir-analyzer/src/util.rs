/// Lowercase a PHP identifier (method name, function name, class name, keyword).
///
/// PHP technically allows bytes `0x80–0xFF` in identifiers, but real-world PHP is
/// overwhelmingly ASCII. `to_ascii_lowercase` is correct for all ASCII identifiers and
/// faster than the Unicode-aware `to_lowercase`; bytes above 0x7F pass through unchanged.
///
/// **Do not use** for docblock content, string literals, or arbitrary source text —
/// those may contain non-ASCII characters that require full Unicode case folding.
#[inline]
pub(crate) fn php_ident_lowercase(s: &str) -> String {
    s.to_ascii_lowercase()
}
