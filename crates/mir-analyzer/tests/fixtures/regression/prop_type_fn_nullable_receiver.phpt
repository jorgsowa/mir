===description===
`is_null($obj->prop)`/`is_string($obj->prop)` etc. on a nullable receiver
must not mark a branch unreachable — `$obj->prop` itself evaluates to
`null` (PHP 8 warning) when `$obj` is null, which is an extra value the
property's own declared type doesn't account for. Uses the
`@mir-check $_ is never` reachability-probe pattern.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
class Holder {
    public string $name = "x";
}

function isNullTrueBranchReachableOnNullableReceiver(?Holder $h): void {
    if (is_null($h->name)) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function isNullTrueBranchDivergesOnNonNullableReceiver(Holder $h): void {
    if (is_null($h->name)) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function isStringFalseBranchReachableOnNullableReceiver(?Holder $h): void {
    if (!is_string($h->name)) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function isStringFalseBranchDivergesOnNonNullableReceiver(Holder $h): void {
    if (!is_string($h->name)) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}
===expect===
PossiblyNullPropertyFetch@7:16-7:24: Cannot access property $name on possibly null value
TypeCheckMismatch@9:8-9:15: Type of $_ is expected to be never, got mixed
RedundantCondition@14:8-14:25: Condition is always true/false for type 'bool'
PossiblyNullPropertyFetch@21:19-21:27: Cannot access property $name on possibly null value
TypeCheckMismatch@23:8-23:15: Type of $_ is expected to be never, got mixed
RedundantCondition@28:8-28:28: Condition is always true/false for type 'bool'
