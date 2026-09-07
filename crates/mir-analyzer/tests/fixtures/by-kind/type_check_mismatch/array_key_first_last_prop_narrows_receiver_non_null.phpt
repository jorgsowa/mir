===description===
`array_key_first($obj->prop) op null`/`array_key_last(...)` narrows the
receiver non-null in both branches — array_key_first(null)/
array_key_last(null) throw a TypeError, so reaching either comparison
result proves the receiver was non-null. narrow_prop_array_key_first_or_last_null
never called narrow_receiver_non_null_on_prop_match at all.
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch,PossiblyNullArgument,MissingPropertyType
===file===
<?php
class Box {
    /** @var array */
    public $items = [];
}

function keyFirstNonNullNarrowsReceiver(?Box $b): void {
    if (array_key_first($b->items) !== null) {
        /** @mir-check $b is Box */
        $_ = 1;
    }
}

function keyFirstNullAlsoNarrowsReceiver(?Box $b): void {
    if (array_key_first($b->items) === null) {
        /** @mir-check $b is Box */
        $_ = 1;
    }
}

function keyLastNonNullNarrowsReceiver(?Box $b): void {
    if (array_key_last($b->items) !== null) {
        /** @mir-check $b is Box */
        $_ = 1;
    }
}
===expect===
