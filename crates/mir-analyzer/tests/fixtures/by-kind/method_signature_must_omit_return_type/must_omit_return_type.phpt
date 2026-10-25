===description===
Must omit return type
===file===
<?php
class A
{
    public function __construct(): void
//                                 ^^^^ CompileError: Method A::__construct() cannot declare a return type [magic-method-return-type]
    {
    }
}
