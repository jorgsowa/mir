===description===
differentArgumentNames
===file===
<?php
                    class A {
                        public function fooFoo(int $a, bool $b): void {

                        }
                    }

                    class B extends A {
                        public function fooFoo(int $a, bool $c): void {

                        }
                    }
===expect===
ParamNameMismatch
===ignore===
TODO
