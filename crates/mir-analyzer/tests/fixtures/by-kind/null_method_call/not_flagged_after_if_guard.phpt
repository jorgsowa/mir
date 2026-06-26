===description===
PossiblyNullMethodCall does NOT fire when a null check guards the method call.
===file===
<?php
class Foo { public function bar(): void {} }
function test(?Foo $obj): void {
    if ($obj !== null) {
        $obj->bar();
    }
}
===expect===
