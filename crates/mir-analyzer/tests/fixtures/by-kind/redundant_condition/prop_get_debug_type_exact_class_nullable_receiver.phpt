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
//                     ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $obj on possibly null value
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior.
function notFooOnNonNullableReceiverDiverges(Holder $h): void {
    if (get_debug_type($h->obj) !== 'Foo') {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        echo "unreachable";
    }
}
===expect===
