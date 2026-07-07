===description===
@psalm-readonly must be recognized as an alias of @readonly, the same way
other psalm-/phpstan- prefixed tags already alias their bare form.
===config===
suppress=MissingConstructor
===file===
<?php
class Foo {
    /** @psalm-readonly */
    public string $name;
}

function setName(Foo $foo, string $name): void {
    $foo->name = $name;
}
===expect===
ReadonlyPropertyAssignment@8:4-8:22: Cannot assign to readonly property Foo::$name outside of constructor
