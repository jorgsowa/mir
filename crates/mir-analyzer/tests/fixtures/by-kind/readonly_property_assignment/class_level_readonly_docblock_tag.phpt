===description===
A class-level `@readonly` docblock tag (a whole-class shorthand,
distinct from the native `readonly class` keyword) makes every own
property readonly, the same as tagging each property individually — but
the per-property `is_readonly` computation never consulted the
class-level docblock's own `is_readonly` flag.
===config===
suppress=MissingConstructor
===file===
<?php
/** @readonly */
class Foo {
    public string $name;
}

function setName(Foo $foo, string $name): void {
    $foo->name = $name;
}
===expect===
ReadonlyPropertyAssignment@8:4-8:22: Cannot assign to readonly property Foo::$name outside of constructor
