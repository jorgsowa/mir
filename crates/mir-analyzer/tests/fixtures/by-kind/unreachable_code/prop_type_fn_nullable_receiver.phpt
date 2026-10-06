===description===
`is_null($obj->prop)`/`is_string($obj->prop)` etc. on a nullable receiver
must not mark a branch unreachable — `$obj->prop` itself evaluates to
`null` (PHP 8 warning) when `$obj` is null, which is an extra value the
property's own declared type doesn't account for. Uses the
`@mir-check $_ is never` reachability-probe pattern.
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
    public string $name = "x";
}

function isNullTrueBranchReachableOnNullableReceiver(?Holder $h): void {
    if (is_null($h->name)) {
//              ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $name on possibly null value
        /** @mir-check $_ is never */
        $_ = 1;
//      ^^^^^^^ TypeCheckMismatch: Type of $_ is expected to be never, got mixed
    }
}

function isNullTrueBranchDivergesOnNonNullableReceiver(Holder $h): void {
    if (is_null($h->name)) {
//      ^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function isStringFalseBranchReachableOnNullableReceiver(?Holder $h): void {
    if (!is_string($h->name)) {
//                 ^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $name on possibly null value
        /** @mir-check $_ is never */
        $_ = 1;
//      ^^^^^^^ TypeCheckMismatch: Type of $_ is expected to be never, got mixed
    }
}

function isStringFalseBranchDivergesOnNonNullableReceiver(Holder $h): void {
    if (!is_string($h->name)) {
//      ^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
        /** @mir-check $_ is never */
        $_ = 1;
    }
}
