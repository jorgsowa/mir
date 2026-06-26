===description===
PossiblyNullMethodCall fires when calling a method on the return value of a
function that returns a nullable type without a null guard.
===file===
<?php
class Foo { public function bar(): void {} }
function maybeNull(): ?Foo { return null; }
function test(): void {
    $x = maybeNull();
    $x->bar();
}
===expect===
PossiblyNullMethodCall@6:4-6:13: Cannot call method bar() on possibly null value
