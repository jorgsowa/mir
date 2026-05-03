===description===
missingTraitPropertyType
===file===
<?php
                    trait T {
                        public $foo = 5;
                    }

                    class A {
                        use T;
                    }
===expect===
MissingPropertyType
===ignore===
TODO
