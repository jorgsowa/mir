===description===
reconcileAfterClassInstanceof
===file===
<?php
                    interface Base {}

                    class E implements Base {
                        public function bar() : void {}
                    }

                    function foobar(Base $foo) : void {
                        if ($foo instanceof E) {
                            $foo->bar();
                        }

                        $foo->bar();
                    }
===expect===
UndefinedInterfaceMethod
===ignore===
TODO
