===description===
Possibly nullable property assignment
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

$a = rand(0, 10) ? new Foo() : null;

$a->foo = "hello";
===expect===
