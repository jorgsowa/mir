===file===
<?php
class Base {
    public function __get(string $name): mixed {
        return null;
    }
}
class Child extends Base {}
function test(): void {
    $c = new Child();
    echo $c->anything;
    echo $c->anotherUndefined;
}
===expect===
