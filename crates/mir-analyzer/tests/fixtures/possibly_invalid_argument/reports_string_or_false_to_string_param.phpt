===file===
<?php
function takesString(string $s): void { var_dump($s); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesString(getResult());
}
===expect===
PossiblyInvalidArgument: Argument $s of takesString() expects 'string', possibly different type 'string|false' provided
