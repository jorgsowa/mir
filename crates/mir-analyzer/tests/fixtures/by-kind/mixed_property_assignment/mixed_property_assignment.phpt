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
//<^^^^^^^^^^^^^^^^^ MixedPropertyAssignment: Property $foo assigned on mixed type
===expect===
