===description===
undefinedCallableMethodFullString
===file===
<?php
                    class A {
                        public static function bar(string $a): string {
                            return $a . "b";
                        }
                    }

                    function foo(callable $c): void {}

                    foo("A::barr");
===expect===
UnusedParam@8:33: Parameter $c is never used
