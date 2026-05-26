===description===
Invalid argument
===file===
<?php
#[Attribute]
class Foo
{
    public function __construct(int $i)
    {
    }
}

#[Foo("foo")]
class Bar{}
===expect===
InvalidScalarArgument
===ignore===
TODO
