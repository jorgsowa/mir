===description===
`is_null($obj->prop) === false` (and the bare `!is_null(...)` form) narrows
the receiver non-null, the `is_null`-specific counterpart of
`narrow_receiver_non_null_on_prop_match` already wired into the literal-
comparison arms — `narrow_prop_from_type_fn` narrowed the property's own
value but never the receiver.
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch
===file===
<?php
class Box {
    public ?string $flag = null;
}

function strictComparison(?Box $x): void {
    if (is_null($x->flag) === false) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function bareNegation(?Box $x): void {
    if (!is_null($x->flag)) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function trueBranchDoesNotProveReceiverNonNull(?Box $x): void {
    if (is_null($x->flag)) {
        // A null receiver also reads $x->flag as null, so this branch
        // doesn't distinguish "receiver null" from "receiver non-null with
        // a null-valued property" — $x must stay nullable here.
        /** @mir-check $x is ?Box */
        $_ = 1;
    }
}
===expect===
