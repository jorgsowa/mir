===description===
PHP 7: non-numeric literal string vs a non-zero literal int is always false.
PHP 7 converts the string to int(0); 0 != 3, so this is impossible in all versions.
===config===
php_version=7.4
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "foo";
    if ($s == 3) {}
}
===expect===
ImpossibleLooseComparison@4:8-4:15: '==' between '"foo"' and '3' is always false — these types can never be loosely equal
