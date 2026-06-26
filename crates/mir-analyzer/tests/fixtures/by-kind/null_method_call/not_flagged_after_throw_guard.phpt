===description===
PossiblyNullMethodCall does NOT fire when a throw-based guard narrows the type
to non-null before the method call.
===file===
<?php
class Foo { public function bar(): void {} }
function test(?Foo $obj): void {
    if ($obj === null) {
        throw new \InvalidArgumentException('obj required');
    }
    $obj->bar();
}
===expect===
