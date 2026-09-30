===description===
Missing attribute on property
===file===
<?php
class Baz
{
    #[Pure]
//    ^^^^ UndefinedAttributeClass: Attribute class Pure does not exist
    public string $foo = "bar";
}

===expect===
