===description===
Mixed property assignment
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

/** @var mixed */
$a = (new Foo());

$a->foo = "hello";
===expect===
MixedPropertyAssignment
===ignore===
TODO
