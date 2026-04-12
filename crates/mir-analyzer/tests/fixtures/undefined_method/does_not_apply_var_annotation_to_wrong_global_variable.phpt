===source===
<?php
class Foo {
    public function bar(): void {}
}

function test(): void {
    /** @var Foo $x */
    global $x, $y;
    $x->bar();
    $y->bar();
}
===expect===
MixedMethodCall: $y->bar()
