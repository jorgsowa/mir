===source===
<?php
class Base {
    public string $name = "";
}
class Child extends Base {}
function test(): void {
    $c = new Child();
    echo $c->name;
}
===expect===
