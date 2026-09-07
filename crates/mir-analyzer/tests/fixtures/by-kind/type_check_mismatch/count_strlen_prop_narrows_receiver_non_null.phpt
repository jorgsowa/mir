===description===
`count($obj->prop) op N`/`strlen($obj->prop) op N` narrows the receiver
non-null — count() throws on a non-Countable (including null) so either
branch proves it, while strlen(null) returns 0 so only the non-empty
branch proves it.
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch,PossiblyNullArgument,MissingPropertyType
===file===
<?php
class Bag {
    /** @var array */
    public $items = [];
    public string $name = '';
}

function countNonEmptyNarrowsReceiver(?Bag $x): void {
    if (count($x->items) > 0) {
        /** @mir-check $x is Bag */
        $_ = 1;
    }
}

function countEmptyAlsoNarrowsReceiver(?Bag $x): void {
    // count() on null is a TypeError, so reaching this comparison at all —
    // in either direction — already proves $x was non-null.
    if (count($x->items) === 0) {
        /** @mir-check $x is Bag */
        $_ = 1;
    }
}

function strlenNonEmptyNarrowsReceiver(?Bag $x): void {
    if (strlen($x->name) > 0) {
        /** @mir-check $x is Bag */
        $_ = 1;
    }
}

function strlenEmptyDoesNotNarrowReceiver(?Bag $x): void {
    // strlen(null) returns 0 without throwing, so a null receiver also
    // satisfies this branch — $x must stay nullable.
    if (strlen($x->name) === 0) {
        /** @mir-check $x is ?Bag */
        $_ = 1;
    }
}
===expect===
