===description===
invalidArgumentWithDeclareStrictTypes
===file===
<?php declare(strict_types=1);
                    function fooFoo(int $a): void {}
                    fooFoo("string");
===expect===
InvalidArgument
===ignore===
TODO
