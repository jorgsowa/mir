===description===
deprecatedPropertyGetAttr
===file===
<?php
                    class A{
                        /**
                         * @var ?int
                         */
                        #[Deprecated]
                        public $foo;
                    }
                    echo (new A)->foo;
===expect===
DeprecatedProperty
===ignore===
TODO
