===description===
readonlyPropertySetInConstructorAndAlsoOutsideClass
===file===
<?php
                    class A {
                        public readonly string $bar;

                        public function __construct() {
                            $this->bar = "hello";
                        }
                    }

                    $a = new A();
                    $a->bar = "goodbye";
===expect===
InaccessibleProperty
===ignore===
TODO
