===description===
implicitCast
===file===
<?php
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
ImplicitToStringCast
===ignore===
TODO
