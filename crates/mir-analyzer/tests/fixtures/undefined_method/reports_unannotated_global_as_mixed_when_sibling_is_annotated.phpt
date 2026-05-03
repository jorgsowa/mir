===description===
reports unannotated global as mixed when sibling is annotated
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(): void {
    /** @var Foo $x */
    global $x;
    global $y;
    $x->bar();
    $y->bar();
}
===expect===
MixedMethodCall: Method bar() called on mixed type
===ignore===
TODO
