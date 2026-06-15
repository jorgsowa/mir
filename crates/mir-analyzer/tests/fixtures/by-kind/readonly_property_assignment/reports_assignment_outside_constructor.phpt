===description===
reports assignment outside constructor
===file===
<?php
class Foo {
    public readonly string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}

function test(Foo $foo): void {
    $foo->name = 'bar';
}
===expect===
ReadonlyPropertyAssignment@11:4-11:22: Cannot assign to readonly property Foo::$name outside of constructor
