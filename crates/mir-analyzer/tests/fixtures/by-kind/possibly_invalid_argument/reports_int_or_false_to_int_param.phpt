===description===
reports int or false to int param
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesInt(getResult());
}
===expect===
PossiblyInvalidArgument@6:13-6:24: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
