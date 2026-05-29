===description===
Invalid argument with declare strict types
===file===
<?php declare(strict_types=1);
                    function fooFoo(int $a): void {}
                    fooFoo("string");
===expect===
UnusedParam@2:37-2:43: Parameter $a is never used
InvalidArgument@3:28-3:36: Argument $a of fooFoo() expects 'int', got '"string"'
