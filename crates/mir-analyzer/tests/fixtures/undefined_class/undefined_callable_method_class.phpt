===description===
undefinedCallableMethodClass
===file===
<?php
                    class A {
                        public static function bar(string $a): string {
                            return $a . "b";
                        }
                    }

                    function foo(callable $c): void {}

                    foo("B::bar");
===expect===
UndefinedClass
===ignore===
TODO
