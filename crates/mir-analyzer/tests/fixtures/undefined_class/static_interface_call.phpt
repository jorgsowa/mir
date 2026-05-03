===description===
staticInterfaceCall
===file===
<?php
                    interface Foo {
                        public static function doFoo();
                    }

                    Foo::doFoo();
===expect===
UndefinedClass
===ignore===
TODO
