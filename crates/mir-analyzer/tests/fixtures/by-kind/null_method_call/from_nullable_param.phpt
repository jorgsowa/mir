===description===
PossiblyNullMethodCall fires when calling a method on a nullable parameter
without a null guard.
===file===
<?php
class Foo { public function bar(): void {} }
function test(?Foo $obj): void {
    $obj->bar();
}
===expect===
PossiblyNullMethodCall@4:4-4:15: Cannot call method bar() on possibly null value
