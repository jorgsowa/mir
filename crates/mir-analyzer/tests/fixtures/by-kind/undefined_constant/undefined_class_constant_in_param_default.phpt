===description===
Undefined class constant in param default
===file===
<?php
class A {
    public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
//                              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedParam: Parameter $howManyTimes is never used
//                                                  ^^^^^^^^^^^^^^^^^^^ UndefinedConstant: Constant A::DEFAULT_TIMES is not defined
}
