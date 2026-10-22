===description===
__construct() cannot declare any return type, not just void; int triggers the same ParseError.
===file===
<?php
class A
{
    public function __construct(): int
//                                 ^^^ CompileError: Compile error: Method A::__construct() cannot declare a return type [magic-method-return-type]
    {
    }
}
