===description===
PHP 8: non-numeric literal string vs positive-int is always false.
No positive integer can ever loosely equal a non-numeric string in PHP 8+.
===config===
php_version=8.0
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test(int $n): void {
    $s = "foo";
    if ($s == $n) {}
}
===expect===
ImpossibleLooseComparison@5:8-5:16: '==' between '"foo"' and 'positive-int' is always false — these types can never be loosely equal
