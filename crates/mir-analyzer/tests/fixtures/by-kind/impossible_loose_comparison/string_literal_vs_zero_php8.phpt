===description===
PHP 8 changed string-vs-int comparison: when the string is non-numeric, the int
is converted to string rather than the string to int.  "foo" == 0 was always true
in PHP 7 (non-numeric string -> int(0)), but is always false in PHP 8+.
===config===
php_version=8.0
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "foo";
    if ($s == 0) {}
}
===expect===
ImpossibleLooseComparison@4:8-4:15: '==' between '"foo"' and '0' is always false — these types can never be loosely equal
