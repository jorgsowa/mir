===description===
cross file child calls removed parent method
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
Child.php: UndefinedMethod@5:5: Method Child::foo() does not exist
