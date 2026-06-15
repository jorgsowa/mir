===description===
Possibly nullable property fetch
===config===
suppress=MissingPropertyType
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

$a = rand(0, 10) ? new Foo() : null;

echo $a->foo;
===expect===
PossiblyNullPropertyFetch@9:5-9:12: Cannot access property $foo on possibly null value
