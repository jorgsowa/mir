===description===
tooFewArgumentsToInstance
===file===
<?php
                    class A {
                        public function fooFoo(int $a): void {}
                    }

                    (new A)->fooFoo();
===expect===
TooFewArguments
===ignore===
TODO
