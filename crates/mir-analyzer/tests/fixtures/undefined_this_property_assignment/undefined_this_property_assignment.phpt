===description===
undefinedThisPropertyAssignment
===file===
<?php
                    class A {
                        public function fooFoo(): void {
                            $this->foo = "cool";
                        }
                    }
===expect===
UndefinedThisPropertyAssignment
===ignore===
TODO
