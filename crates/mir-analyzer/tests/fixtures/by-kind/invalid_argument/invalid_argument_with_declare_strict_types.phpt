===description===
Invalid argument with declare strict types
===file===
<?php declare(strict_types=1);
                    function fooFoo(int $a): void {}
//                                  ^^^^^^ UnusedParam: Parameter $a is never used
                    fooFoo("string");
//                         ^^^^^^^^ InvalidArgument: Argument $a of fooFoo() expects 'int', got '"string"'
===expect===
