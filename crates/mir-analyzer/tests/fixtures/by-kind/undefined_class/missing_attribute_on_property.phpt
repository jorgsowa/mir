===description===
Missing attribute on property
===file===
<?php
class Baz
{
    #[Pure]
    public string $foo = "bar";
}

===expect===
UndefinedAttributeClass@4:7-4:11: Attribute class Pure does not exist
