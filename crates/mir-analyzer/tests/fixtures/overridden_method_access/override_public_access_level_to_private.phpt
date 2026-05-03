===description===
overridePublicAccessLevelToPrivate
===file===
<?php
                    class A {
                        public function fooFoo(): void {}
                    }

                    class B extends A {
                        private function fooFoo(): void {}
                    }
===expect===
OverriddenMethodAccess
===ignore===
TODO
