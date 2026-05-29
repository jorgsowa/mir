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
PossiblyNullMethodCall@6:5-6:16: Cannot call method bar() on possibly null value
