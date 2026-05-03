===description===
does not report var annotated global inside function
===file===
<?php
class Foo {
    public function bar(): void {}
}

function test(): void {
    /** @var Foo $x */
    global $x;
    $x->bar();
}
===expect===
===ignore===
TODO
