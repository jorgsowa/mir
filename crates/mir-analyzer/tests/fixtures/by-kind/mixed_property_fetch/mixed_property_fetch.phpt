===description===
Mixed property fetch
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

echo $a->foo;
===expect===
MixedPropertyFetch@10:6-10:13: Property $foo fetched on mixed type
