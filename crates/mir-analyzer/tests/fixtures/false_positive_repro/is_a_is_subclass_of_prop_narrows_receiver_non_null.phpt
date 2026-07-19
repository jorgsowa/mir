===description===
`is_a($obj->prop, X::class)` / `is_subclass_of($obj->prop, X::class)` /
`get_parent_class($obj->prop) === X::class` (which reuses the same
is_subclass_of narrowing) all proved the property but never called
narrow_receiver_non_null_on_prop_match, unlike the direct `$x instanceof
Y` arm — $obj itself stayed nullable even though a true result is only
possible when $obj->prop is a real (non-null) value, which requires $obj
itself to be non-null.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyNullPropertyFetch,PossiblyNullArgument
===file===
<?php
class Base {}
class Derived extends Base {}

final class Holder {
    /** @var Derived|null */
    public $child;
}

function isANarrowsReceiver(?Holder $h): void {
    if (is_a($h->child, Base::class)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function isSubclassOfNarrowsReceiver(?Holder $h): void {
    if (is_subclass_of($h->child, Base::class)) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function getParentClassNarrowsReceiver(?Holder $h): void {
    if (get_parent_class($h->child) === Base::class) {
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

function isSubclassOfFalseBranchDoesNotNarrowReceiver(?Holder $h): void {
    if (!is_subclass_of($h->child, Base::class)) {
        /** @mir-check $h is Holder|null */
        $_ = 1;
    }
}
===expect===
