===description===
cannotOverrideFinalType
===file===
<?php
                    class P {
                        public final function f() : void {}
                    }

                    class C extends P {
                        public function f() : void {}
                    }
===expect===
MethodSignatureMismatch
===ignore===
TODO
