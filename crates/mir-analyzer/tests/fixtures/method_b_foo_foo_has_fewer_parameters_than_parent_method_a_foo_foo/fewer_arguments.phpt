===description===
fewerArguments
===file===
<?php
                    class A {
                        public function fooFoo(int $a, bool $b): void {

                        }
                    }

                    class B extends A {
                        public function fooFoo(int $a): void {

                        }
                    }
===expect===
Method B::fooFoo has fewer parameters than parent method A::fooFoo
===ignore===
TODO
