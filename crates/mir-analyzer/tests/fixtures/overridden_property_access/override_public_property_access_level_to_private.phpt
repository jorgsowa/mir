===description===
overridePublicPropertyAccessLevelToPrivate
===file===
<?php
                    class A {
                        /** @var string|null */
                        public $foo;
                    }

                    class B extends A {
                        /** @var string|null */
                        private $foo;
                    }
===expect===
OverriddenPropertyAccess
===ignore===
TODO
