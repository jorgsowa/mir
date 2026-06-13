===description===
DirectConstructorCall does NOT fire for normal object instantiation.
===file===
<?php
class Foo {
    public function __construct(int $x) {}
}

$obj = new Foo(1);
===expect===
