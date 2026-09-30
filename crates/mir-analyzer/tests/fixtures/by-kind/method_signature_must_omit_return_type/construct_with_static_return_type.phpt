===description===
__construct() cannot declare a static return type (PHP 8 late static binding).
===file===
<?php
class A
{
    public function __construct(): static
//                                 ^^^^^^ ParseError: Parse error: Method __construct() cannot declare a return type
    {
    }
}
===expect===
