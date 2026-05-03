===description===
overridePublicPropertyAccessLevelToProtected
===file===
<?php
                    class A {
                        /** @var string|null */
                        public $foo;
                    }

                    class B extends A {
                        /** @var string|null */
                        protected $foo;
                    }
===expect===
OverriddenPropertyAccess
===ignore===
TODO
