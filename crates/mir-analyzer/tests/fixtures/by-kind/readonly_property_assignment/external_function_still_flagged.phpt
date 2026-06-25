===description===
ReadonlyPropertyAssignment still fires when an external (non-class) function assigns to a readonly property
===config===
suppress=MissingConstructor
===file===
<?php
class Foo {
    public readonly string $name;
}

function setName(Foo $foo, string $name): void {
    $foo->name = $name;
}
===expect===
ReadonlyPropertyAssignment@7:4-7:22: Cannot assign to readonly property Foo::$name outside of constructor
