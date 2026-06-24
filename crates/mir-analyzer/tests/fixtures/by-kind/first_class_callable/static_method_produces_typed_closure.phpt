===description===
P3: Static method first-class callable Cls::method(...) produces a typed Closure.
===config===
suppress=UnusedVariable
===file===
<?php

class Math {
    public static function double(int $n): int {
        return $n * 2;
    }
}

$fn = Math::double(...);
/** @mir-check $fn is Closure(int): int */
$_ = $fn;
===expect===
