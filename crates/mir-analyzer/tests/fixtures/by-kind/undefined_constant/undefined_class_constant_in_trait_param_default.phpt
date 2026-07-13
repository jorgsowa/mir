===description===
FN: trait method param default-value expressions were never analyzed at
all, unlike the equivalent class method — only param count/type checks
ran, so an undefined global constant default went unflagged. (`self::`
is deliberately exempt inside a trait — late static binding may resolve
it on the using class — so this uses a global constant instead.)
===file===
<?php
trait T {
    public function doSomething(int $howManyTimes = UNDEFINED_CONST): void {}
}
===expect===
UnusedParam@3:32-3:67: Parameter $howManyTimes is never used
UndefinedConstant@3:52-3:67: Constant UNDEFINED_CONST is not defined
