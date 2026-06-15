===description===
reports string or false to string param
===config===
suppress=ForbiddenCode
===file===
<?php
function takesString(string $s): void { var_dump($s); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesString(getResult());
}
===expect===
PossiblyInvalidArgument@6:16-6:27: Argument $s of takesString() expects 'string', possibly different type 'string|false' provided
