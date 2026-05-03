===description===
magicMethodOverridesParentWithDifferentReturnType
===file===
<?php
                    class C {}
                    class D {}

                    class A {
                        public function foo(string $s) : C {
                            return new C;
                        }
                    }

                    /** @method D foo(string $s) */
                    class B extends A {}
===expect===
ImplementedReturnTypeMismatch
===ignore===
TODO
