===description===
`array_key_exists('k', $h->data)` throws TypeError on a null 2nd arg, so
reaching EITHER branch already proves $h->data (and thus $h) was non-null —
but the prop arm never called narrow_receiver_non_null_on_prop_match like
every sibling arm (class_exists, str_contains, in_array, ...) does.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyNullArgument,PossiblyNullPropertyFetch
===file===
<?php
final class Holder {
    /** @var array */
    public $data = [];
}

function narrowsReceiverTrueBranch(?Holder $h): void {
    if (array_key_exists('id', $h->data)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function narrowsReceiverFalseBranch(?Holder $h): void {
    if (!array_key_exists('id', $h->data)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}
===expect===
