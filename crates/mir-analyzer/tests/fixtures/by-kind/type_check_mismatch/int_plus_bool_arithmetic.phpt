===description===
FP: int + bool (e.g. `$i += 2 + isset($x)`) should infer int, not int|float.
PHP always coerces bool→int in arithmetic; the result is never float.
===config===
suppress=UnusedVariable,UnusedParam
php_version=8.4
===file===
<?php

function test_isset(array $arr): void {
    $i = 0;
    $i += 2 + isset($arr['k']);
    /** @mir-check $i is int */
    $_ = $i;
}

function test_bool_param(bool $flag): void {
    $n = 1 + $flag;
    /** @mir-check $n is int */
    $_ = $n;
}

function test_bool_plus_bool(bool $a, bool $b): void {
    $n = $a + $b;
    /** @mir-check $n is int */
    $_ = $n;
}

function test_null_plus_int(int $n): void {
    $v = null + $n;
    /** @mir-check $v is int */
    $_ = $v;
}
===expect===
