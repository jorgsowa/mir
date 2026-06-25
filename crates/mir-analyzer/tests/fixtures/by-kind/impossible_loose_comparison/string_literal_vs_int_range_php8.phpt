===description===
PHP 8: non-numeric literal string vs int<1, 100> is always false.
No integer in the range [1, 100] can loosely equal a non-numeric string in PHP 8+.
===config===
php_version=8.0
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<1, 100> $n */
function test(int $n): void {
    $s = "baz";
    if ($s == $n) {}
}
===expect===
ImpossibleLooseComparison@5:8-5:16: '==' between '"baz"' and 'int<1, 100>' is always false — these types can never be loosely equal
