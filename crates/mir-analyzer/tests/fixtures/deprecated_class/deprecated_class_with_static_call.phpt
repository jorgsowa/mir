===description===
deprecatedClassWithStaticCall
===file===
<?php
                    /**
                     * @deprecated
                     */
                    class Foo {
                        public static function barBar(): void {
                        }
                    }

                    Foo::barBar();
===expect===
DeprecatedClass
===ignore===
TODO
