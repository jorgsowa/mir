===description===
reports both null and false independently
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|null|false */
function getResult(): int|null|false { return 1; }
function test(): void {
    takesInt(getResult());
}
===expect===
PossiblyInvalidArgument@6:13-6:24: Argument $n of takesInt() expects 'int', possibly different type 'int|null|false' provided
PossiblyNullArgument@6:13-6:24: Argument $n of takesInt() might be null
