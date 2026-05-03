===description===
warnAboutMismatchingClassParamDoc
===file===
<?php
                    class A {}
                    class B {}

                    class X {
                        /**
                         * @param B $class
                         */
                        public function boo(A $class): void {}
                    }
===expect===
MismatchingDocblockParamType
===ignore===
TODO
