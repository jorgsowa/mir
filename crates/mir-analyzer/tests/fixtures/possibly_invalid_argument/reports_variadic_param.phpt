===description===
reports variadic param
===file===
<?php
function takesInts(int ...$ns): void { var_dump($ns); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesInts(getResult(), getResult());
}
===expect===
PossiblyInvalidArgument@6:14: Argument $ns of takesInts() expects 'int', possibly different type 'int|false' provided
PossiblyInvalidArgument@6:27: Argument $ns of takesInts() expects 'int', possibly different type 'int|false' provided
