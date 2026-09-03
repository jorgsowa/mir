===description===
`$obj->prop > N` / `>= N` / `< N` / `<= N` proved the property's own
narrowed range but never called narrow_receiver_non_null_on_prop_match
even when int_comparison_excludes_null proves the comparison couldn't
have been satisfied by a null receiver — $obj itself stayed nullable.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType,PossiblyNullPropertyFetch
===file===
<?php
final class Node {
    /** @var int|null */
    public $value;
}

function greaterThanNonzeroNarrowsReceiver(?Node $n): void {
    if ($n->value > 5) {
        /** @mir-check $n is Node */
        $_ = 1;
    }
}

function greaterOrEqualNonzeroNarrowsReceiver(?Node $n): void {
    if ($n->value >= 5) {
        /** @mir-check $n is Node */
        $_ = 1;
    }
}

function greaterOrEqualZeroDoesNotNarrowReceiver(?Node $n): void {
    if ($n->value >= 0) {
        /** @mir-check $n is Node|null */
        $_ = 1;
    }
}
===expect===
