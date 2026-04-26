===file===
<?php
class Base {
    public function __call(string $name, array $arguments): mixed {
        return null;
    }
}
class Child extends Base {}
function test(): void {
    $c = new Child();
    $c->anything();
    $c->anotherMissing(1, 2);
}
===expect===
