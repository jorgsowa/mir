===description===
does not report var annotated global variable
===file===
<?php
class Foo {
    public function bar(): void {}
}

/** @var Foo $x */
global $x;
$x->bar();
===expect===
===ignore===
TODO
