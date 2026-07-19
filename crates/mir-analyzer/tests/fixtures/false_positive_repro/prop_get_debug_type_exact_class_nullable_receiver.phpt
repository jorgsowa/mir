===description===
`get_debug_type($obj->prop) !== 'Foo'` (where `prop`'s declared type is the
single final class `Foo`) on a nullable `$obj` receiver must not mark the
branch unreachable — `get_debug_type(null)` returns the string `'null'`,
which is never `'Foo'`, so a nullable receiver can make the comparison true
regardless of the property's own precise declared type. Non-nullable
receivers keep diverging on a genuine contradiction.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
final class Foo {}
class Holder {
    public Foo $obj;
}

// Positive: reachable when $h is null (get_debug_type(null) is 'null').
function notFooOnNullableReceiverReachable(?Holder $h): void {
    if (get_debug_type($h->obj) !== 'Foo') {
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior.
function notFooOnNonNullableReceiverDiverges(Holder $h): void {
    if (get_debug_type($h->obj) !== 'Foo') {
        echo "unreachable";
    }
}
===expect===
PossiblyNullPropertyFetch@9:23-9:30: Cannot access property $obj on possibly null value
RedundantCondition@16:8-16:41: Condition is always true/false for type 'bool'
