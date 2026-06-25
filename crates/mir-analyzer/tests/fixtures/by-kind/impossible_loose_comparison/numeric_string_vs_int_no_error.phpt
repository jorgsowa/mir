===description===
Conservative: numeric literal strings ("123", "3.14") can loosely equal integers/floats.
PHP 8 compares them numerically when the string is numeric — no warning.
===config===
php_version=8.0
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "123";
    if ($s == 123) {}
    $t = "3.14";
    if ($t == 3) {}
    $u = "0";
    if ($u == 0) {}
}
===expect===
