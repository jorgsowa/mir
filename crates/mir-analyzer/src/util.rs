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

/// Real global-namespace PHP classes/interfaces commonly referenced bare in
/// **docblocks** with no explicit `use` import. Unlike a pseudo-type keyword
/// (`array`/`iterable`/…), these are actual classes — PHP's own namespace
/// resolution rules would require an import or a leading `\` for them in
/// real code, but Psalm/PHPStan resolve bare docblock references to them
/// leniently, so mir does too rather than mis-qualifying them against the
/// current namespace (the same failure mode already fixed for `iterable`'s
/// implicit `Traversable` member).
///
/// Only for docblock-type resolution — do not use this for resolving a real
/// code reference (a type hint, `instanceof`, `Foo::class`, …), where a bare
/// unqualified name genuinely does need the current namespace prepended,
/// same as real PHP.
///
/// `Countable` is deliberately excluded: unlike these others, it's common
/// enough as a user-redeclared interface name (a local `Countable` in a
/// legacy/polyfill namespace) that treating it as always-global would shadow
/// a real same-namespace declaration.
pub(crate) fn is_global_builtin_docblock_class(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "closure"
            | "traversable"
            | "iterator"
            | "iteratoraggregate"
            | "arrayaccess"
            | "generator"
            | "stringable"
            | "stdclass"
            | "throwable"
    )
}
