===description===
FN: enum method param default-value expressions were never analyzed at
all, unlike the equivalent class method — only param count/type checks
ran, so an undefined class constant default went unflagged.
===file===
<?php
enum Suit {
    case Hearts;

    public function doSomething(int $howManyTimes = self::DEFAULT_TIMES): void {}
}
===expect===
UnusedParam@5:32-5:71: Parameter $howManyTimes is never used
UndefinedConstant@5:52-5:71: Constant Suit::DEFAULT_TIMES is not defined
