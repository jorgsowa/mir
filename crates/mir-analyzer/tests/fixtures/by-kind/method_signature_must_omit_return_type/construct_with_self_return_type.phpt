===description===
__construct() cannot declare a self return type.
===file===
<?php
class A
{
    public function __construct(): self
//                                 ^^^^ ParseError: Parse error: Method A::__construct() cannot declare a return type [magic-method-return-type]
    {
    }
}
