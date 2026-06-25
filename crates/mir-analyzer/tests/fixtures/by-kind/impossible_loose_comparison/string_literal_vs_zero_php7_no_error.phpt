===description===
PHP 7: non-numeric string == 0 is NOT impossible — it is actually true.
PHP 7 converts the string to int(0) and compares, so "foo" == 0 evaluates to true.
No warning should be emitted.
===config===
php_version=7.4
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "foo";
    if ($s == 0) {}
}
===expect===
