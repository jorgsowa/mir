===description===
selfClassConstBadConst
===file===
<?php
                    class A {
                        const FOO = "foo";
                        const BAR = "bar";

                        /**
                         * @param (self::1FOO | self::BAR) $s
                         */
                        public static function foo(string $s) : void {}
                    }
===expect===
InvalidDocblock
===ignore===
TODO
