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
//      ^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $cls on possibly null value
        /** @mir-check $_ is never */
        $_ = 1;
//      ^^^^^^^ TypeCheckMismatch: Type of $_ is expected to be never, got mixed
    }
}

function classStringExclusionDivergesOnNonNullableReceiver(ClsHolder $h): void {
    if ($h->cls !== Foo::class) {
//      ^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        /** @mir-check $_ is never */
        $_ = 1;
    }
}

function enumCaseExclusionReachableOnNullableReceiver(?StatusHolder $h): void {
    if ($h->status !== Status::Active) {
//      ^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $status on possibly null value
        /** @mir-check $_ is never */
        $_ = 1;
//      ^^^^^^^ TypeCheckMismatch: Type of $_ is expected to be never, got mixed
    }
}

function enumCaseExclusionDivergesOnNonNullableReceiver(StatusHolder $h): void {
    if ($h->status !== Status::Active) {
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
        /** @mir-check $_ is never */
        $_ = 1;
    }
}
===expect===
