===description===
Invalid argument
===config===
suppress=UnusedParam
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
