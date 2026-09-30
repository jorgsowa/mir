===description===
reports named argument
===config===
suppress=ForbiddenCode
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesInt(n: getResult());
//           ^^^^^^^^^^^^^^ PossiblyInvalidArgument: Argument $n of takesInt() expects 'int', possibly different type 'int|false' provided
}
===expect===
