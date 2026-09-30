===description===
does not report when param accepts bool
===config===
suppress=ForbiddenCode
===file===
<?php
function takesBool(bool $b): void { var_dump($b); }
/** @return int|false */
function getResult(): int|false { return 1; }
function test(): void {
    takesBool(getResult());
//            ^^^^^^^^^^^ PossiblyInvalidArgument: Argument $b of takesBool() expects 'bool', possibly different type 'int|false' provided
}
===expect===
