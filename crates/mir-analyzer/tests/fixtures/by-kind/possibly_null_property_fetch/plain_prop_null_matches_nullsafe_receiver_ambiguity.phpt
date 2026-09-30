===description===
Plain `$obj->prop` null narrowing must match `?->`'s receiver ambiguity —
PHP 8 reads a `->` access on a null receiver as a warning, not fatal,
still evaluating to null. Non-nullable receivers keep the old behavior
(see prop_null_and_enum_case_contradiction_diverges.phpt).
===config===
suppress=UnusedVariable,MissingConstructor
===file===
<?php
class Foo {
    public string $bar = 'x';
}

// Positive: reachable when $obj is null, so not RedundantCondition.
function nullCaseReachable(?Foo $obj): void {
    if ($obj->bar === null) {
//      ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $bar on possibly null value
        echo "reached when \$obj is null";
    }
}

class Box {
    public ?string $value = null;
    public string $other = "x";
}

// Positive: proving $value non-null also proves the receiver non-null.
function receiverNarrowed(?Box $b): void {
    if ($b->value !== null) {
//      ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $value on possibly null value
        echo $b->other;
    }
}
===expect===
