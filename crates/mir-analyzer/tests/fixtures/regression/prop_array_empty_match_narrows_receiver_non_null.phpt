===description===
Proving `$obj->prop === []` (or loose `== []`) must also prove `$obj` itself
is non-null, same reasoning as the literal/bool/int/enum-case cases already
fixed — `null !== []` always, so a matched-true comparison rules out a null
receiver. The excluded/false direction proves nothing.
===config===
suppress=UnusedParam,MissingConstructor
===file===
<?php
class Foo {
    /** @var array<int> */
    public array $items = [];

    public function ping(): void {}
}

function viaStrictEmpty(?Foo $foo): void {
    if ($foo->items === []) {
        $foo->ping();
    }
}

function viaLooseEmpty(?Foo $foo): void {
    if ($foo->items == []) {
        $foo->ping();
    }
}

// Negative: the excluded branch proves nothing about $foo itself.
function viaStrictNotEmptyFalseBranch(?Foo $foo): void {
    if (!($foo->items === [])) {
        $foo->ping();
    }
}
===expect===
PossiblyNullPropertyFetch@10:8-10:19: Cannot access property $items on possibly null value
PossiblyNullPropertyFetch@16:8-16:19: Cannot access property $items on possibly null value
PossiblyNullPropertyFetch@23:10-23:21: Cannot access property $items on possibly null value
PossiblyNullMethodCall@24:8-24:20: Cannot call method ping() on possibly null value
