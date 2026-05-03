===description===
noInstanceCallAsStatic
===file===
<?php
                    class C {
                        public function foo() : void {}
                    }

                    (new C)::foo();
===expect===
InvalidStaticInvocation
===ignore===
TODO
