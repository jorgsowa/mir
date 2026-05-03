===description===
parentStaticCall
===file===
<?php
                    class A {
                        /** @return void */
                        public function foo(){}
                    }

                    class B extends A {
                        /** @return void */
                        public static function bar(){
                            parent::foo();
                        }
                    }
===expect===
InvalidStaticInvocation
===ignore===
TODO
