===description===
__construct() cannot declare any return type, not just void; int triggers the same ParseError.
===file===
<?php
class A
{
    public function __construct(): int
    {
    }
}
===expect===
ParseError@4:35-4:38: Parse error: Method __construct() cannot declare a return type
