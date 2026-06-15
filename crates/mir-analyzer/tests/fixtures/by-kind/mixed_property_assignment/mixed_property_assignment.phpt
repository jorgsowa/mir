===description===
Mixed property assignment
===config===
suppress=MissingPropertyType
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
MixedPropertyAssignment@10:0-10:17: Property $foo assigned on mixed type
