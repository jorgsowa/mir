===description===
undefinedClassConstantInParamDefault
===file===
<?php
                    class A {
                        public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
                    }
===expect===
UndefinedConstant
===ignore===
TODO
