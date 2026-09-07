===description===
`@psalm-assert-if-true`/`-if-false` used as an `if` condition on a
property-access argument narrows the receiver non-null too, mirroring every
other narrowing arm in this file — `apply_docblock_assertions` set the
property's own refined type but never proved the receiver itself wasn't
null.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyNullPropertyFetch,PossiblyNullArgument
===file===
<?php
class Bar {}

final class Holder {
    /** @var Bar|null */
    public $child;
}

/** @psalm-assert-if-true Bar $x */
function isBar(mixed $x): bool {
    return $x instanceof Bar;
}

/** @psalm-assert-if-false Bar $x */
function isNotBar(mixed $x): bool {
    return !($x instanceof Bar);
}

function trueBranchNarrowsReceiver(?Holder $h): void {
    if (isBar($h->child)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function falseBranchNarrowsReceiver(?Holder $h): void {
    if (!isNotBar($h->child)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function outsideBranchDoesNotNarrow(?Holder $h): void {
    if (isBar($h->child)) {
        $_ = 1;
    }
    /** @mir-check $h is Holder|null */
    $_ = 1;
}
===expect===
