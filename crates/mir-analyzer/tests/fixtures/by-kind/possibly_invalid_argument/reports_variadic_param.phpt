===description===
reports variadic param
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInts(int ...$ns): void { var_dump($ns); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesInts(getResult(), getResult());
}
===expect===
PossiblyInvalidArgument@6:15-6:26: Argument $ns of takesInts() expects 'int', possibly different type 'int|false' provided
PossiblyInvalidArgument@6:28-6:39: Argument $ns of takesInts() expects 'int', possibly different type 'int|false' provided
