===description===
deprecatedCloneMethodWithCall
===file===
<?php
                    class Foo {
                        /**
                         * @deprecated
                         */
                        public function __clone() {
                        }
                    }

                    $a = new Foo;
                    $aa = clone $a;
===expect===
DeprecatedMethod
===ignore===
TODO
