===description===
possiblyNullablePropertyFetch
===file===
<?php
class Foo {
    /** @var string */
    public $foo = "";
}

$a = rand(0, 10) ? new Foo() : null;

echo $a->foo;
===expect===
PossiblyNullPropertyFetch
===ignore===
TODO
