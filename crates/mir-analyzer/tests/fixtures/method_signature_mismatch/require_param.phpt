===description===
requireParam
===file===
<?php
                    interface I {
                        function foo(bool $b = false): void;
                    }

                    class C implements I {
                        public function foo(bool $b): void {}
                    }
===expect===
MethodSignatureMismatch
===ignore===
TODO
