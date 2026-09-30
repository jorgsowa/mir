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
//   ^^^^^^^ MixedPropertyFetch: Property $foo fetched on mixed type
===expect===
