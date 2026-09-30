===description===
PossiblyNullMethodCall fires when calling a method on a nullable parameter
without a null guard.
===file===
<?php
class Foo { public function bar(): void {} }
function test(?Foo $obj): void {
    $obj->bar();
//  ^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method bar() on possibly null value
}
===expect===
