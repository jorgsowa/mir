===description===
`in_array($obj->prop, [...])` narrows the receiver non-null on a true
match — a strict match can never match a null receiver's coerced-to-null
property (the haystack never contains a literal null element), and a
loose match only can when the haystack contains a falsy literal (0, "",
"0"), which the fix excludes.
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch,MissingPropertyType
===file===
<?php
class Box {
    public $status;
}

function strictMatchNarrowsReceiver(?Box $b): void {
    if (in_array($b->status, ['a', 'b'], true)) {
        /** @mir-check $b is Box */
        $_ = 1;
    }
}

function looseMatchAgainstNonFalsyLiteralsNarrowsReceiver(?Box $b): void {
    if (in_array($b->status, ['a', 'b'])) {
        /** @mir-check $b is Box */
        $_ = 1;
    }
}

function looseMatchAgainstFalsyLiteralDoesNotNarrowReceiver(?Box $b): void {
    // haystack contains "" — null loosely equals "", so a true match here
    // doesn't rule out $b being null.
    if (in_array($b->status, ['', 'a'])) {
        /** @mir-check $b is ?Box */
        $_ = 1;
    }
}

function looseMatchAgainstZeroDoesNotNarrowReceiver(?Box $b): void {
    // haystack contains 0 — null loosely equals 0.
    if (in_array($b->status, [0, 5])) {
        /** @mir-check $b is ?Box */
        $_ = 1;
    }
}
===expect===
