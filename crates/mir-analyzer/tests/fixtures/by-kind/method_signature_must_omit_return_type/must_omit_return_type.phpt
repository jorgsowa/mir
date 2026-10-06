===description===
Must omit return type
===file===
<?php
class A
{
    public function __construct(): void
//                                 ^^^^ ParseError: Parse error: Method __construct() cannot declare a return type
    {
    }
}
