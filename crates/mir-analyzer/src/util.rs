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

/// Every native PHP superglobal name (without the `$` prefix), for purity
/// checks that treat reading/writing one as touching external mutable
/// state — deliberately broader than `taint::SUPERGLOBALS` (which excludes
/// `$_SESSION`/`GLOBALS`/`argv`/`argc` since those aren't attacker-controlled
/// taint sources; purity cares about "is this external state", not "is this
/// user input", so the two lists have different membership on purpose).
pub(crate) fn is_superglobal_name(name: &str) -> bool {
    matches!(
        name,
        "GLOBALS"
            | "_SERVER"
            | "_GET"
            | "_POST"
            | "_REQUEST"
            | "_SESSION"
            | "_COOKIE"
            | "_FILES"
            | "_ENV"
    )
}
