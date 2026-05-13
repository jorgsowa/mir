===description===
implicitCastWithStrictTypes
===file===
<?php declare(strict_types=1);
                    class A {
                        public function __toString(): string
                        {
                            return "hello";
                        }
                    }

                    /** @psalm-mutation-free */
                    function fooFoo(string $b): void {}
                    fooFoo(new A());
===expect===
UnusedParam@10:36: Parameter $b is never used
InvalidArgument@11:27: Argument $b of fooFoo() expects 'string', got 'A'
