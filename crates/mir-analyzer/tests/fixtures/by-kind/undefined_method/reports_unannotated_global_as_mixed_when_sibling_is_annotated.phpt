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
MixedMethodCall@11:4-11:13: Method bar() called on mixed type
