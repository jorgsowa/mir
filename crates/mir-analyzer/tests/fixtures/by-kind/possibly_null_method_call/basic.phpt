===description===
Basic
===file===
<?php
class Foo {
    public function bar(): void {}
}
function test(?Foo $obj): void {
    $obj->bar();
//  ^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method bar() on possibly null value
}
