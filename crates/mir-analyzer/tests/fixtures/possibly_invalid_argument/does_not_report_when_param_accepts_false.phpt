===file===
<?php
function takesIntOrFalse(int|false $n): void { var_dump($n); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesIntOrFalse(getResult());
}
===expect===
