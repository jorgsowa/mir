===description===
clearDependentTypeWhenAssigning
===file===
<?php
                    class A {}

                    class AChild extends A {
                        public function bar() : void {}
                    }

                    class B {}

                    function foo(A $a) : void {
                        $a_class = get_class($a);

                        $a = new B();

                        switch ($a_class) {
                            case AChild::class:
                                $a->bar();
                        }
                    }
===expect===
UndefinedMethod@17:32: Method B::bar() does not exist
