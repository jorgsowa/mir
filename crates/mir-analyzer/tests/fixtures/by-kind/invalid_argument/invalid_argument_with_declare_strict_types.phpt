===description===
Invalid argument with declare strict types
===file===
<?php declare(strict_types=1);
                    function fooFoo(int $a): void {}
                    fooFoo("string");
===expect===
UnusedParam@2:36-2:42: Parameter $a is never used
InvalidArgument@3:27-3:35: Argument $a of fooFoo() expects 'int', got '"string"'
