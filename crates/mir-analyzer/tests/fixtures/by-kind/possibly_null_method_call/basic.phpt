===description===
Basic
===file===
<?php
class Foo {
    public function bar(): void {}
}
function test(?Foo $obj): void {
    $obj->bar();
}
===expect===
PossiblyNullMethodCall@6:4-6:15: Cannot call method bar() on possibly null value
