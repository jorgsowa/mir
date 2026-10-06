===description===
A class-level `@readonly` docblock tag (a whole-class shorthand,
distinct from the native `readonly class` keyword) makes every own
property readonly, the same as tagging each property individually — but
the per-property `is_readonly` computation never consulted the
class-level docblock's own `is_readonly` flag.
===config===
<mir>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @readonly */
class Foo {
    public string $name;
}

function setName(Foo $foo, string $name): void {
    $foo->name = $name;
//  ^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Foo::$name outside of constructor
}
