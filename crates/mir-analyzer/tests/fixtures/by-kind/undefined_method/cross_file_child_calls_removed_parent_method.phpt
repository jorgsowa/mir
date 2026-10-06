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
//  ^^^^^^^^^ UndefinedMethod: Method Child::foo() does not exist
}
