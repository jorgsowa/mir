===description===
reports string or false to string param
===file===
<?php
function takesString(string $s): void { var_dump($s); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesString(getResult());
}
===expect===
PossiblyInvalidArgument@6:17-6:28: Argument $s of takesString() expects 'string', possibly different type 'string|false' provided
