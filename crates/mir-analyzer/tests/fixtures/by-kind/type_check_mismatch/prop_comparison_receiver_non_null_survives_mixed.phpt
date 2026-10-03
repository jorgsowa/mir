===description===
Int-comparison/count()/strlen() property comparisons narrow the receiver
non-null even when the property's own type is mixed — the receiver-non-null
call used to sit after an `is_mixed()` early return, so a mixed-typed
property lost the receiver reasoning entirely.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <PossiblyNullPropertyFetch errorLevel="suppress"/>
    <PossiblyNullArgument errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//            ^^^^^^^^^ MixedArgument: Argument $value of count() is mixed
        /** @mir-check $x is Box */
        $_ = 1;
    }
}

function strlenNonEmptyNarrowsReceiverEvenWhenMixed(?Box $x): void {
    if (strlen($x->name) > 0) {
//             ^^^^^^^^ MixedArgument: Argument $string of strlen() is mixed
        /** @mir-check $x is Box */
        $_ = 1;
    }
}
===expect===
