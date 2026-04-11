===source===
<?php
class Foo {
    public function bar(): void {}
}
function test(?Foo $obj): void {
    $obj->bar();
}
===expect===
PossiblyNullMethodCall: $obj->bar()
