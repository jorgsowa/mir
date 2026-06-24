===description===
P3: Static method first-class callable preserves all parameter types.
===config===
suppress=UnusedVariable
===file===
<?php

class StringHelper {
    public static function pad(string $s, int $len, string $char): string {
        return str_pad($s, $len, $char);
    }
}

$fn = StringHelper::pad(...);
/** @mir-check $fn is Closure(string, int, string): string */
$_ = $fn;
===expect===
