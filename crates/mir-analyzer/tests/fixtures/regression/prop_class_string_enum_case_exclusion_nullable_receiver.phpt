===description===
`$obj->prop !== Foo::class` / `$obj->prop !== Status::Active` on a nullable
`$obj` receiver must not mark the exclusion branch unreachable — a null
`$obj` makes `$obj->prop` itself evaluate to `null`, which is never `===`
a class-string or enum-case literal, regardless of the property's own
declared type. Uses the `@mir-check $_ is never` reachability probe.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
final class Foo {}
class ClsHolder {
    /** @var class-string<Foo> */
    public string $cls;
}

enum Status { case Active; }
class StatusHolder {
    public Status $status;
}

function classStringExclusionReachableOnNullableReceiver(?ClsHolder $h): void {
    if ($h->cls !== Foo::class) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function classStringExclusionDivergesOnNonNullableReceiver(ClsHolder $h): void {
    if ($h->cls !== Foo::class) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function enumCaseExclusionReachableOnNullableReceiver(?StatusHolder $h): void {
    if ($h->status !== Status::Active) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function enumCaseExclusionDivergesOnNonNullableReceiver(StatusHolder $h): void {
    if ($h->status !== Status::Active) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}
===expect===
PossiblyNullPropertyFetch@14:8-14:15: Cannot access property $cls on possibly null value
TypeCheckMismatch@16:8-16:15: Type of $_ is expected to be never, got mixed
RedundantCondition@21:8-21:30: Condition is always true/false for type 'bool'
PossiblyNullPropertyFetch@28:8-28:18: Cannot access property $status on possibly null value
TypeCheckMismatch@30:8-30:15: Type of $_ is expected to be never, got mixed
RedundantCondition@35:8-35:37: Condition is always true/false for type 'bool'
