===description===
undefinedClassConstantInParamDefault
===file===
<?php
                    class A {
                        public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
                    }
===expect===
UnusedParam@3:52: Parameter $howManyTimes is never used
UndefinedConstant@3:72: Constant A::DEFAULT_TIMES is not defined
