===description===
Type narrowing through if/elseif/else branches. `is_bool($value)` in the
third arm is genuinely redundant (L19 fix): after excluding int and string,
$value can only be bool there — previously masked because each elseif
re-derived its context from the primary if's condition alone, discarding
the prior elseif's own narrowing.
===config===
suppress=MissingReturnType
===file===
<?php
function testBranches(int|string|bool $value) {
    if (is_int($value)) {
        /** @mir-check $value is int */
        return $value + 1;
    } elseif (is_string($value)) {
        /** @mir-check $value is string */
        return strlen($value);
    } elseif (is_bool($value)) {
        /** @mir-check $value is bool */
        return $value ? 'true' : 'false';
    }
    return null;
}

function testMultipleBranches(int|string|float|null $x) {
    if ($x === null) {
        /** @mir-check $x is null */
        return 'null';
    } elseif (is_int($x)) {
        /** @mir-check $x is int */
        return $x + 1;
    } elseif (is_string($x)) {
        /** @mir-check $x is string */
        return strlen($x);
    } else {
        /** @mir-check $x is float */
        return $x + 1.5;
    }
}

function testElseAfterNarrowing(string|null $value) {
    if (!is_null($value)) {
        /** @mir-check $value is string */
        return strlen($value);
    } else {
        /** @mir-check $value is null */
        return null;
    }
}
===expect===
RedundantCondition@9:14-9:29: Condition is always true/false for type 'bool'
UnreachableCode@13:4-13:16: Unreachable code detected
