===description===
A nested array-offset write (`$arr[$k1][$k2] = …`) only coerced the
OUTERMOST index's key to PHP's canonical array-key form (bool -> 0/1,
float truncates, null -> ""); the inner walk-up loop's key stayed
uncoerced, so a dynamic bool/float/null key anywhere but the last bracket
kept its raw type instead.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function nestedBoolKeyCoercesToInt(bool $flag): void {
    $arr = [];
    $arr[$flag]['x'] = 1;
    /** @mir-check $arr is array<int, array<"x", 1>> */
    $_ = $arr;
}

function nestedFloatKeyCoercesToInt(float $f): void {
    $arr = [];
    $arr[$f]['x'] = 1;
    /** @mir-check $arr is array<int, array<"x", 1>> */
    $_ = $arr;
}

function nestedNullKeyCoercesToEmptyString(): void {
    $arr = [];
    $n = null;
    $arr[$n]['x'] = 1;
    /** @mir-check $arr is array<string, array<"x", 1>> */
    $_ = $arr;
}
===expect===
