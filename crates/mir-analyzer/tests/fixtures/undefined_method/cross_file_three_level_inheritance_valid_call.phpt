===file:GrandParent.php===
<?php
class GrandParent {
    public function greet(): void {}
}
===file:Middle.php===
<?php
class Middle extends GrandParent {}
===file:Child.php===
<?php
class Child extends Middle {}
function test(): void {
    $c = new Child();
    $c->greet();
}
===expect===
