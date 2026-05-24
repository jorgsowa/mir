===description===
reports bool union to non bool param
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|bool */
function getResult(): int|bool { return 1; }
function test(): void {
    takesInt(getResult());
}
===expect===
PossiblyInvalidArgument@6:14: Argument $n of takesInt() expects 'int', possibly different type 'int|bool' provided
