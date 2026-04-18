===source===
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
ReadonlyPropertyAssignment: $foo->name = 'bar'
