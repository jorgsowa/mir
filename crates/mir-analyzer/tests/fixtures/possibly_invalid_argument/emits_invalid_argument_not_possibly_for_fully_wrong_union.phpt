===description===
emits invalid argument not possibly for fully wrong union
===file===
<?php
function takesInt(int $n): void { var_dump($n); }
/** @return string|false */
function getResult(): string|false { return 'x'; }
function test(): void {
    takesInt(getResult());
}
===expect===
InvalidArgument@6:13: Argument $n of takesInt() expects 'int', got 'string|false'
===ignore===
TODO
