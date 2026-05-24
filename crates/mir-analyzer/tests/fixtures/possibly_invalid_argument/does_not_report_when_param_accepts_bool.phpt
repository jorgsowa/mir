===description===
does not report when param accepts bool
===file===
<?php
function takesBool(bool $b): void { var_dump($b); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesBool(getResult());
}
===expect===
InvalidArgument@6:15: Argument $b of takesBool() expects 'bool', got 'int|false'
