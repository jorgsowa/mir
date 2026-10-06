===description===
`!($obj->prop instanceof X)` / `!is_a($obj->prop, X::class)` on a nullable
receiver must not mark the branch unreachable — `null instanceof X` is
always false, so a nullable $obj can make the false branch true regardless
of the property's own declared type. Non-nullable receivers keep diverging
on a genuine contradiction. The true branch was already sound.
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
class Bar {}
class Holder {
    public Bar $prop;
}

// Positive: reachable when $h is null.
function instanceofFalseOnNullableReceiverReachable(?Holder $h): void {
    if (!($h->prop instanceof Bar)) {
//        ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $prop on possibly null value
        /** @mir-check $h->prop is Bar|null */
        $_ = 1;
    }
}

function isAFalseOnNullableReceiverReachable(?Holder $h): void {
    if (!is_a($h->prop, Bar::class)) {
//            ^^^^^^^^ PossiblyNullArgument: Argument $object_or_class of is_a() might be null
//            ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $prop on possibly null value
        /** @mir-check $h->prop is Bar|null */
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior.
function instanceofFalseOnNonNullableReceiverDiverges(Holder $h): void {
    if (!($h->prop instanceof Bar)) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo "unreachable";
    }
}

function isAFalseOnNonNullableReceiverDiverges(Holder $h): void {
    if (!is_a($h->prop, Bar::class)) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        echo "unreachable";
    }
}
