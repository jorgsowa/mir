===description===
readonlyPropertyWithDefault
===file===
<?php
                    class A {
                        public readonly string $s = "a";
                    }
===expect===
InvalidPropertyAssignment
===ignore===
TODO
