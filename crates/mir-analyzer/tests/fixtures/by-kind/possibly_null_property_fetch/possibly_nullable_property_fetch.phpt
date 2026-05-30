===description===
Possibly nullable property fetch
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

$a = rand(0, 10) ? new Foo() : null;

echo $a->foo;
===expect===
PossiblyNullPropertyFetch@9:6-9:13: Cannot access property $foo on possibly null value
