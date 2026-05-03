===description===
missingPropertyTypeWithConstructorInitAndNull
===file===
<?php
                    class A {
                        public $foo;

                        public function __construct() {
                            $this->foo = 5;
                        }

                        public function makeNull(): void {
                            $this->foo = null;
                        }
                    }
===expect===
MissingPropertyType
===ignore===
TODO
