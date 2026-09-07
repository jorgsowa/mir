===description===
Int-comparison/count()/strlen() property comparisons narrow the receiver
non-null even when the property's own type is mixed — the receiver-non-null
call used to sit after an `is_mixed()` early return, so a mixed-typed
property lost the receiver reasoning entirely.
===config===
suppress=UnusedVariable,PossiblyNullPropertyFetch,PossiblyNullArgument,MissingPropertyType
===file===
<?php
class Box {
    public $value;
    /** @var mixed */
    public $items;
    /** @var mixed */
    public $name;
}

function intComparisonNarrowsReceiverEvenWhenMixed(?Box $x): void {
    if ($x->value > 5) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function countComparisonNarrowsReceiverEvenWhenMixed(?Box $x): void {
    if (count($x->items) > 0) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function strlenNonEmptyNarrowsReceiverEvenWhenMixed(?Box $x): void {
    if (strlen($x->name) > 0) {
        /** @mir-check $x is Box */
        $_ = 1;
    }
}
===expect===
MixedArgument@18:14-18:23: Argument $value of count() is mixed
MixedArgument@25:15-25:23: Argument $string of strlen() is mixed
