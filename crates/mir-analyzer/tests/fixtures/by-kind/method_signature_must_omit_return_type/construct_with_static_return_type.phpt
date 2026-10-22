===description===
__construct() cannot declare a static return type (PHP 8 late static binding).
===file===
<?php
class A
{
    public function __construct(): static
//                                 ^^^^^^ CompileError: Compile error: Method A::__construct() cannot declare a return type [magic-method-return-type]
    {
    }
}
