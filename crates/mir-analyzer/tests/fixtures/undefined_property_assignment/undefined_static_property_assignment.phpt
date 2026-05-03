===description===
undefinedStaticPropertyAssignment
===file===
<?php
                    class A {
                        public static function barBar(): void
                        {
                            /** @psalm-suppress UndefinedPropertyFetch */
                            self::$foo = 5;
                        }
                    }
===expect===
UndefinedPropertyAssignment
===ignore===
TODO
