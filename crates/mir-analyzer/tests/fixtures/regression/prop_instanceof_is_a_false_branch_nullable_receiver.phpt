===description===
`!($obj->prop instanceof X)` / `!is_a($obj->prop, X::class)` on a nullable
receiver must not mark the branch unreachable — `null instanceof X` is
always false, so a nullable $obj can make the false branch true regardless
of the property's own declared type. Non-nullable receivers keep diverging
on a genuine contradiction. The true branch was already sound.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Bar {}
class Holder {
    public Bar $prop;
}

// Positive: reachable when $h is null.
function instanceofFalseOnNullableReceiverReachable(?Holder $h): void {
    if (!($h->prop instanceof Bar)) {
        /** @mir-check $h->prop is Bar|null */
        $_ = 1;
    }
}

function isAFalseOnNullableReceiverReachable(?Holder $h): void {
    if (!is_a($h->prop, Bar::class)) {
        /** @mir-check $h->prop is Bar|null */
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior.
function instanceofFalseOnNonNullableReceiverDiverges(Holder $h): void {
    if (!($h->prop instanceof Bar)) {
        echo "unreachable";
    }
}

function isAFalseOnNonNullableReceiverDiverges(Holder $h): void {
    if (!is_a($h->prop, Bar::class)) {
        echo "unreachable";
    }
}
===expect===
PossiblyNullPropertyFetch@9:10-9:18: Cannot access property $prop on possibly null value
PossiblyNullArgument@16:14-16:22: Argument $object_or_class of is_a() might be null
PossiblyNullPropertyFetch@16:14-16:22: Cannot access property $prop on possibly null value
RedundantCondition@24:8-24:34: Condition is always true/false for type 'bool'
RedundantCondition@30:8-30:35: Condition is always true/false for type 'bool'
