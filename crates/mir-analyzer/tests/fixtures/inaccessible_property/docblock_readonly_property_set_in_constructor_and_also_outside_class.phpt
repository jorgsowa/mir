===description===
docblockReadonlyPropertySetInConstructorAndAlsoOutsideClass
===file===
<?php
                    class A {
                        /**
                         * @readonly
                         */
                        public string $bar;

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
