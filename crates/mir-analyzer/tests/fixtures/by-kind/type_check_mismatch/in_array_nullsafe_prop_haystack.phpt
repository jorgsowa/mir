===description===
`extract_haystack_type`'s match arm explicitly listed
`NullsafePropertyAccess` but called `extract_prop_access` (plain `->`
only), which never matches it — the arm was dead code, so
`in_array($x, $h?->tags, true)` never narrowed the needle.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Holder {
    /** @var array{0: 'a', 1: 'b', 2: 'c'} */
    public $tags = ['a', 'b', 'c'];
}

function narrowsNeedleViaNullsafeHaystack(Holder $h, string $x): void {
    if (in_array($x, $h?->tags, true)) {
        /** @mir-check $x is 'a'|'b'|'c' */
        $_ = $x;
    }
}

function narrowsNeedleViaPlainArrowStillWorks(Holder $h, string $x): void {
    if (in_array($x, $h->tags, true)) {
        /** @mir-check $x is 'a'|'b'|'c' */
        $_ = $x;
    }
}
===expect===
