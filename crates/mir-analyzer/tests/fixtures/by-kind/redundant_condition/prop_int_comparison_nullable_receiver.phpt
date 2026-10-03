===description===
`$obj->prop < N` (or `<=`/`>`/`>=`) on a nullable receiver must not mark the
branch unreachable — PHP's ordering-comparison table converts a null
receiver's property read and the int literal to bool and compares those,
which can make the comparison true regardless of the property's own
precise, out-of-range declared type. Non-nullable receivers keep diverging
on a genuine contradiction.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//      ^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $level on possibly null value
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior.
function lessThanOnNonNullableReceiverDiverges(Holder $h): void {
    if ($h->level < -5) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo "unreachable";
    }
}
===expect===
