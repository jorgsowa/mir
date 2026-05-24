===description===
undefinedClassConstantInParamDefault
===file===
<?php
class A {
    public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
}
===expect===
UnusedParam@3:33: Parameter $howManyTimes is never used
UndefinedConstant@3:53: Constant A::DEFAULT_TIMES is not defined
