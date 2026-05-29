===description===
Undefined class constant in param default
===file===
<?php
class A {
    public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
}
===expect===
UnusedParam@3:33-3:72: Parameter $howManyTimes is never used
UndefinedConstant@3:53-3:72: Constant A::DEFAULT_TIMES is not defined
