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
MixedPropertyAssignment@10:1-10:18: Property $foo assigned on mixed type
