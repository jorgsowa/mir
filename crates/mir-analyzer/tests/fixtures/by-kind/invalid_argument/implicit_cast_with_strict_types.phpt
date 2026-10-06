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
//                                  ^^^^^^^^^ UnusedParam: Parameter $b is never used
                    fooFoo(new A());
//                         ^^^^^^^ InvalidArgument: Argument $b of fooFoo() expects 'string', got 'A'
