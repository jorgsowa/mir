===description===
reconcileAfterInterfaceInstanceof
===file===
<?php
                    interface Base {}

                    interface E extends Base {
                        public function bar() : void;
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
