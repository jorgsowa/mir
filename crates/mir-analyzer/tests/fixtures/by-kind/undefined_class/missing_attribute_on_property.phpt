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
UndefinedAttributeClass@4:6-4:10: Attribute class Pure does not exist
