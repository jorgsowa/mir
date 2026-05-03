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
MixedMethodCall@10:4: Method bar() called on mixed type
===ignore===
TODO
