===description===
__construct() cannot declare any return type, not just void; int triggers the same ParseError.
===file===
<?php
class A
{
    public function __construct(): int
//                                 ^^^ ParseError: Parse error: Method __construct() cannot declare a return type
    {
    }
}
