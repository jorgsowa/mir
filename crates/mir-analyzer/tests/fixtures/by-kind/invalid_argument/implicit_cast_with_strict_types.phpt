===description===
Implicit cast with strict types
===file===
<?php declare(strict_types=1);
                    class A {
                        public function __toString(): string
                        {
                            return "hello";
                        }
                    }

                    /** @mutation-free */
                    function fooFoo(string $b): void {}
                    fooFoo(new A());
===expect===
UnusedParam@10:36-10:45: Parameter $b is never used
InvalidArgument@11:27-11:34: Argument $b of fooFoo() expects 'string', got 'A'
