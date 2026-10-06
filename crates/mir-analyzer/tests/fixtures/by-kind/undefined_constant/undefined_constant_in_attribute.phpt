===description===
Undefined constant in attribute
===file===
<?php
#[Attribute]
class Foo
{
    public function __construct(int $i) {}
//                              ^^^^^^ UnusedParam: Parameter $i is never used
}

#[Foo(self::BAR_CONST)]
class Bar {}
                
