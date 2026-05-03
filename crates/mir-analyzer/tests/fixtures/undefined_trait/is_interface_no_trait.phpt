===description===
isInterfaceNoTrait
===file===
<?php
                    Interface B {}

                    class A {
                        use B;
                    }
===expect===
UndefinedTrait
===ignore===
TODO
