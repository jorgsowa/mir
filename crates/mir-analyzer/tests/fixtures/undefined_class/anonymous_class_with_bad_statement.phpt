===description===
anonymousClassWithBadStatement
===file===
<?php
                    $foo = new class {
                        public function a() {
                            new B();
                        }
                    };
===expect===
UndefinedClass
===ignore===
TODO
