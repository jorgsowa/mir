===description===
Loose counterpart of plain_prop_null_matches_nullsafe_receiver_ambiguity.phpt:
`$obj->prop == null` on a nullable receiver must not mark the branch
unreachable either — `narrow_prop_loose_null` skipped the nullable-receiver
gate that `narrow_prop_null` already had. Non-nullable receivers keep
diverging on a genuine contradiction.
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

// Positive: reachable when $h is null, so $h->prop keeps its
// receiver-null-ambiguous `Bar|null` type rather than being wrongly emptied.
function looseNullOnNullableReceiverReachable(?Holder $h): void {
    if ($h->prop == null) {
//      ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $prop on possibly null value
        /** @mir-check $h->prop is Bar|null */
        $_ = 1;
    }
}

// Positive: proving `!= null` also proves the receiver itself is non-null.
function looseNotNullProvesReceiverNonNull(?Holder $h): void {
    if ($h->prop != null) {
//      ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $prop on possibly null value
        /** @mir-check $h is Holder */
        $_ = 1;
    }
}

// Negative: a non-nullable receiver keeps the old, sound behavior — an
// object property can never be loose-null-equal, so this stays a genuine
// contradiction.
function looseNullOnNonNullableReceiverDiverges(Holder $h): void {
    if ($h->prop == null) {
//      ^^^^^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'Bar' and 'null' is always false — these types can never be loosely equal
        echo "unreachable";
    }
}
===expect===
