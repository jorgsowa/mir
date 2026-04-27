===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|null|false */
function getResult(): int|null|false { return 1; }
function test(): void {
    takesInt(getResult());
}
===expect===
PossiblyNullArgument: Argument $n of takesInt() might be null
PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|null|false' provided
