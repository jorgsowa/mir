===description===
Negative control: when the enum-typed receiver itself is nullable,
covering every case's `->value` still isn't exhaustive without an
explicit `null` arm — `$type->value` on a null `$type` evaluates to null.
===config===
suppress=UnusedParam
===file===
<?php
enum Kind: string {
    case Foo = 'foo';
    case Bar = 'bar';
}
function h(?Kind $type): bool {
    return match ($type->value) {
        Kind::Foo->value => true,
        Kind::Bar->value => false,
    };
}
===expect===
UnhandledMatchCondition@7:11-10:5: Unhandled match condition: null
PossiblyNullPropertyFetch@7:18-7:30: Cannot access property $value on possibly null value
