===description===
Type narrowing through if/elseif/else branches
===file===
<?php
function testBranches(int|string|bool $value) {
    if (is_int($value)) {
        return $value + 1;
    } elseif (is_string($value)) {
        return strlen($value);
    } elseif (is_bool($value)) {
        return $value ? 'true' : 'false';
    }
    return null;
}

function testMultipleBranches(int|string|float|null $x) {
    if ($x === null) {
        return 'null';
    } elseif (is_int($x)) {
        return $x + 1;
    } elseif (is_string($x)) {
        return strlen($x);
    } else {
        return $x + 1.5;
    }
}

function testElseAfterNarrowing(string|null $value) {
    if (!is_null($value)) {
        return strlen($value);
    } else {
        return null;
    }
}
===expect===
UnusedParam@13:30: Parameter $x is never used
UnusedParam@25:32: Parameter $value is never used
