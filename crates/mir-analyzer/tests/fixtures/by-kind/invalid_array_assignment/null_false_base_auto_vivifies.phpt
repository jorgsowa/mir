===description===
An array write on a null or false base auto-vivifies it into an array. Other
scalar bases are still reported.
===file===
<?php
/** @return list<string> */
function push_on_null_or_false(null|false $s): array {
    $s[] = 'x';
    /** @mir-check $s is list<"x"> */
    return $s;
}

/** @return array<string, int> */
function keyed_on_null_or_false(null|false $s): array {
    $s['k'] = 1;
    /** @mir-check $s is array<string, 1> */
    return $s;
}

/** @return list<string> */
function push_on_false(false $s): array {
    $s[] = 'x';
//  ^^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type 'false'
    return $s;
}

/** @return array<string, int> */
function keyed_on_null(): array {
    $s = null;
    $s['k'] = 1;
    return $s;
}

/** @param array<string, int>|false $s */
function keyed_on_false_or_array(array|false $s): int {
    $s['k'] = 1;
    /** @mir-check $s is array<string, int> */
    return count($s);
}

function push_on_int(int $s): int {
    $s[] = 'x';
//  ^^^^^^^^^^ InvalidArrayAssignment: Cannot use [] assignment on non-array type 'int'
    return $s;
}
===expect===
