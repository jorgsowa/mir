===description===
Undefined class constant in param default
===file===
<?php
class A {
    public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
}
===expect===
UnusedParam@3:32-3:71: Parameter $howManyTimes is never used
UndefinedConstant@3:52-3:71: Constant A::DEFAULT_TIMES is not defined
