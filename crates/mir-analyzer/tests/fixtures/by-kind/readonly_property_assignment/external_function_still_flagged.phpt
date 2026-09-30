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
//  ^^^^^^^^^^^^^^^^^^ ReadonlyPropertyAssignment: Cannot assign to readonly property Foo::$name outside of constructor
}
===expect===
