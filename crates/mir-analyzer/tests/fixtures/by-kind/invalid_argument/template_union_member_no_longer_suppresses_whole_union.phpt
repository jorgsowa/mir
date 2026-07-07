===description===
KNOWN LIMITATION (not correct behavior, pinned so a future fix updates this
fixture deliberately instead of silently changing it): `param_contains_
template_or_unknown` (crates/mir-analyzer/src/call/args.rs) is a plain
`.any()` over a param union's atoms — if ANY alternative mentions an
unresolved template anywhere in its own type args (here `Bar<T>` in
`Foo|Bar<T>`), argument checking is skipped for the ENTIRE union, even
though a candidate argument may satisfy NEITHER alternative. Passing a bare
string here matches neither `Foo` nor any instantiation of `Bar<T>`, yet no
diagnostic fires at all — not even `MixedArgument`.
A real fix needs to check argument-checkability per union alternative
instead of forgiving the whole parameter the moment any one alternative is
still generic, but that risks reintroducing false positives on other
intentionally-lenient template call sites, so it needs its own dedicated,
carefully verified pass rather than a drive-by change.
===config===
suppress=UnusedParam,MissingReturnType
===file===
<?php
class Foo {}

/** @template T */
class Bar {
    /** @param T $x */
    public function __construct($x) {}
}

/**
 * @template T
 * @param Foo|Bar<T> $x
 */
function takesFooOrBar($x): void {}

takesFooOrBar("plain-string");
===expect===
