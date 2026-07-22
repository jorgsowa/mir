===description===
PHP casts non-string/int array keys to their canonical form: bool -> 0/1,
float -> truncated int, null -> "". Both a keyed write (`$arr[key] = …`)
and an array literal (`[key => …]`) must apply the same casting instead of
keeping the raw bool/float/null value as the key type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function writeBoolKey(): void {
    $arr = [];
    $arr[true] = 1;
    /** @mir-check $arr is array{1: 1} */
    $_ = $arr;
}

function writeFloatKey(): void {
    $arr = [];
    $arr[1.9] = 1;
    /** @mir-check $arr is array{1: 1} */
    $_ = $arr;
}

function writeNullKey(): void {
    $arr = [];
    $arr[null] = 1;
    /** @mir-check $arr is array{'': 1} */
    $_ = $arr;
}

function literalBoolKey(): void {
    $arr = [true => 1];
    /** @mir-check $arr is array{1: 1} */
    $_ = $arr;
}

function literalFloatKey(): void {
    $arr = [1.9 => 1];
    /** @mir-check $arr is array{1: 1} */
    $_ = $arr;
}

function literalNullKey(): void {
    $arr = [null => 1];
    /** @mir-check $arr is array{'': 1} */
    $_ = $arr;
}

function dynamicBoolKeyFallsBackToInt(bool $b): void {
    $arr = [$b => 1];
    /** @mir-check $arr is array<int, 1> */
    $_ = $arr;
}
===expect===
ImplicitFloatToIntCast@30:12-30:15: Implicit cast from 1.9 to int truncates the fractional part
