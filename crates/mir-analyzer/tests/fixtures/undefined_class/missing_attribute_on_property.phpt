===description===
Missing attribute on property
===file===
<?php
use FooBarPure;

class Baz
{
    #[Pure]
    public string $foo = "bar";
}

===expect===
UndefinedAttributeClass
===ignore===
TODO
