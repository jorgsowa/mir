===description===
dontReportImplementerErrorOnAbstractTraitMethodTwice
===file===
<?php
                    trait B {
                        abstract public function run();
                    }

                    final class A {
                        use B;

                        #[Override]
                        public function run(string $foo): string {
                            return $foo;
                        }
                    }
===expect===
MethodSignatureMismatch
===ignore===
TODO
