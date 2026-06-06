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
ParseError@2:5-2:15: Parse error: The use statement with non-compound name 'FooBarPure' has no effect
UndefinedAttributeClass@6:7-6:11: Attribute class Pure does not exist
