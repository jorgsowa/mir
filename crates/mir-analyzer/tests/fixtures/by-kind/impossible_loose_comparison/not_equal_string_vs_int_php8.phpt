===description===
PHP 8: "foo" != 0 is always true (the != operator).
The != operator is the inverse of ==; the same impossibility applies.
===config===
php_version=8.0
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $s = "foo";
    if ($s != 0) {}
}
===expect===
ImpossibleLooseComparison@4:8-4:15: '!=' between '"foo"' and '0' is always true — these types can never be loosely equal
