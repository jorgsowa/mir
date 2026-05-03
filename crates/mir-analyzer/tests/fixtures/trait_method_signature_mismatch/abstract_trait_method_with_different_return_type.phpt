===description===
abstractTraitMethodWithDifferentReturnType
===file===
<?php
                    class A {}
                    class B {}

                    trait T {
                        abstract public function foo() : A;
                    }

                    class C {
                        use T;

                        public function foo() : B{
                            return new B();
                        }
                    }
===expect===
TraitMethodSignatureMismatch
===ignore===
TODO
