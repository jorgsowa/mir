===description===
`str_contains($obj->prop, 'x')`/`str_starts_with(...)`/`str_ends_with(...)`
true (with a non-empty literal needle) narrows the receiver non-null — a
null receiver reads the property as null, which coerces to "", and a
non-empty needle can never be found in "".
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch,PossiblyNullArgument
===file===
<?php
class Box {
    public string $name = '';
}

function containsNarrowsReceiver(?Box $x): void {
    if (str_contains($x->name, 'a')) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function startsWithNarrowsReceiver(?Box $x): void {
    if (str_starts_with($x->name, 'a')) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function endsWithNarrowsReceiver(?Box $x): void {
    if (str_ends_with($x->name, 'a')) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function falseBranchDoesNotProveReceiverNonNull(?Box $x): void {
    if (!str_contains($x->name, 'a')) {
        // A null receiver also reads $x->name as "", which doesn't
        // contain 'a' either — the false branch doesn't distinguish
        // "receiver null" from "receiver non-null without a match".
        /** @mir-check $x is ?Box */
        $_ = 1;
    }
}

function emptyNeedleDoesNotProveReceiverNonNull(?Box $x): void {
    if (str_contains($x->name, '')) {
        // Every string (including "" from a null receiver) contains "".
        /** @mir-check $x is ?Box */
        $_ = 1;
    }
}
===expect===
