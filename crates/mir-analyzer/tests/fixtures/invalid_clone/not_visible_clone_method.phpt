===description===
notVisibleCloneMethod
===file===
<?php
                    class A {
                        private function __clone() {}
                    }
                    $a = new A();
                    clone $a;
===expect===
InvalidClone
===ignore===
TODO
