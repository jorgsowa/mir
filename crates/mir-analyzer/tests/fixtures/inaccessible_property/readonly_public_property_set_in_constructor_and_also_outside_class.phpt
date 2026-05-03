===description===
readonlyPublicPropertySetInConstructorAndAlsoOutsideClass
===file===
<?php
                    class A {
                        /**
                         * @psalm-readonly-allow-private-mutation
                         */
                        public string $bar;

                        public function __construct() {
                            $this->bar = "hello";
                        }

                        public function setAgain() : void {
                            $this->bar = "hello";
                        }
                    }

                    $a = new A();
                    $a->bar = "goodbye";
===expect===
InaccessibleProperty
===ignore===
TODO
