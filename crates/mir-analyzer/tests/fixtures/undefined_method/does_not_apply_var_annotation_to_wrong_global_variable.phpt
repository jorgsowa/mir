===description===
does not apply var annotation to wrong global variable
===file===
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
MixedMethodCall: Method bar() called on mixed type
===ignore===
TODO
