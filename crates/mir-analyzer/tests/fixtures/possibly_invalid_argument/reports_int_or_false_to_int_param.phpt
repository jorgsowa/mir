===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesInt(getResult());
}
===expect===
PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
