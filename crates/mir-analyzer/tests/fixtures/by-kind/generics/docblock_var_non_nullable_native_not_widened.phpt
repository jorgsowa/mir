===description===
Negative control for the L28 fix: a `@var` docblock type refining a
NON-nullable native hint must not gain a spurious `|null` — the
nullability-preserving fix only applies when the native hint itself is
nullable.
===config===
suppress=MissingConstructor,UnusedVariable
===file===
<?php
class Foo {}

class Box {
    /** @var Foo */
    public Foo $item;
}

function check(Box $b): void {
    /** @mir-check $b->item is Foo */
    $x = $b->item;
}
===expect===
