===description===
analyzeConstructorForNonexistentAttributes
===file===
<?php
class Foo
{
    public function __construct(string $_arg) {}
}

/** @psalm-suppress UndefinedAttributeClass */
#[AttrA(new Foo(1))]
class Bar {}

===expect===
InvalidScalarArgument
===ignore===
TODO
