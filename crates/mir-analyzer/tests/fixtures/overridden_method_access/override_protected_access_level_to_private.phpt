===description===
overrideProtectedAccessLevelToPrivate
===file===
<?php
                    class A {
                        protected function fooFoo(): void {}
                    }

                    class B extends A {
                        private function fooFoo(): void {}
                    }
===expect===
OverriddenMethodAccess
===ignore===
TODO
