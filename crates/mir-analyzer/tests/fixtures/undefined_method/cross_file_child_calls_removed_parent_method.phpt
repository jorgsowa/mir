===file:Base.php===
<?php
class Base {}
===file:Child.php===
<?php
class Child extends Base {}
function test(): void {
    $c = new Child();
    $c->foo();
}
===expect===
Child.php: UndefinedMethod: Method Child::foo() does not exist
