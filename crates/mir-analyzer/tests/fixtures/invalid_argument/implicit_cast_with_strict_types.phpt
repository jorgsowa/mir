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
InvalidArgument
===ignore===
TODO
