===description===
__construct() cannot declare a self return type.
===file===
<?php
class A
{
    public function __construct(): self
    {
    }
}
===expect===
ParseError@4:35-4:39: Parse error: Method __construct() cannot declare a return type
