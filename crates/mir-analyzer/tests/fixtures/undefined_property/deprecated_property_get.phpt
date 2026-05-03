===description===
deprecatedPropertyGet
===file===
<?php
                    class A{
                        /**
                         * @deprecated
                         * @var ?int
                         */
                        public $foo;
                    }
                    echo (new A)->foo;
===expect===
DeprecatedProperty
===ignore===
TODO
