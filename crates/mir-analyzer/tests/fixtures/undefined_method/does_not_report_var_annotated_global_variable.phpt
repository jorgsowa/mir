===source===
<?php
class Foo {
    public function bar(): void {}
}

/** @var Foo $x */
global $x;
$x->bar();
===expect===
