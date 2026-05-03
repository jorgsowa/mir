===description===
isClassNoTrait
===file===
<?php
                    class B {}

                    class A {
                        use B;
                    }
===expect===
UndefinedTrait
===ignore===
TODO
