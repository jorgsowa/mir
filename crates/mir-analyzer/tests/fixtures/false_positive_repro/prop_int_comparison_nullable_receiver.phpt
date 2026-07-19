===description===
`$obj->prop < N` (or `<=`/`>`/`>=`) on a nullable receiver must not mark the
branch unreachable — PHP's ordering-comparison table converts a null
receiver's property read and the int literal to bool and compares those,
which can make the comparison true regardless of the property's own
precise, out-of-range declared type. Non-nullable receivers keep diverging
on a genuine contradiction.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Holder {
    /** @var int<1, 10> */
    public int $level = 1;
}

// Positive: reachable when $h is null (null->level reads null;
// null < -5 compares as false < true = true).
function lessThanOnNullableReceiverReachable(?Holder $h): void {
    if ($h->level < -5) {
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior.
function lessThanOnNonNullableReceiverDiverges(Holder $h): void {
    if ($h->level < -5) {
        echo "unreachable";
    }
}
===expect===
PossiblyNullPropertyFetch@10:8-10:17: Cannot access property $level on possibly null value
RedundantCondition@17:8-17:22: Condition is always true/false for type 'bool'
