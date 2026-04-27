===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesInt(getResult());
}
===expect===
InvalidArgument: Argument $n of takesInt() expects 'int', got 'string|false'
