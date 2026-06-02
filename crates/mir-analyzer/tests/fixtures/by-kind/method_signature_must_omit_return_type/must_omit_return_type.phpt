===description===
Must omit return type
===file===
<?php
class A
{
    public function __construct(): void
    {
    }
}
===expect===
ParseError@4:36-4:40: Parse error: Method __construct() cannot declare a return type
