===description===
A non-numeric literal string vs a non-zero integer is always false in PHP 8:
the int is converted to string and compared, so "bar" != "5".
===config===
php_version=8.0
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "bar";
    if ($s == 5) {}
}
===expect===
ImpossibleLooseComparison@4:8-4:15: '==' between '"bar"' and '5' is always false — these types can never be loosely equal
