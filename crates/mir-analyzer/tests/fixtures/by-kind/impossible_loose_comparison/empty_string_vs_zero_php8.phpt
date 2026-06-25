===description===
PHP 8: empty string vs 0 is always false.
"" is not a numeric string, so PHP 8 converts 0 to "0"; "" != "0".
In PHP 7 "" == 0 was true (empty string -> int(0)).
===config===
php_version=8.0
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "";
    if ($s == 0) {}
}
===expect===
ImpossibleLooseComparison@4:8-4:15: '==' between '""' and '0' is always false — these types can never be loosely equal
