===description===
undefinedClassConstantInParamDefault
===file===
<?php
                    class A {
                        public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
                    }
===expect===
UnusedParam@3:52: Parameter $howManyTimes is never used
