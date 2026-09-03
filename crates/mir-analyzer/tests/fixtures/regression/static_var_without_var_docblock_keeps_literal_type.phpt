===description===
Negative control for the `static $x = null;` + `@var` fix: with NO preceding `@var`
docblock, a static var's type must still come from its literal initializer as before
— `static $x = null;` alone still makes a later `$x !== null` check genuinely always
false, and that diagnostic must keep firing.
===config===
suppress=UnusedParam
===file===
<?php
function test(): void {
    static $x = null;
    if ($x !== null) {
        echo 'unreachable';
    }
}
===expect===
RedundantCondition@4:8-4:19: Condition is always true/false for type 'bool'
