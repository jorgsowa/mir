===description===
callOnVoid
===file===
<?php
                    class A {
                        public function foo(): void {}
                    }

                    $p = new A();
                    $p->foo()->bar();
===expect===
NullReference
===ignore===
TODO
