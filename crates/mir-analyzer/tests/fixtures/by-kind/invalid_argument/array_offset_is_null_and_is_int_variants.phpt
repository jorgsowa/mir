===description===
`is_int()` and `!is_null()` narrow array-offset types.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array{v: string|int} $arr */
function testIsInt(array $arr): int {
    if (is_int($arr['v'])) {
        return $arr['v'] + 1;
    }
    return 0;
}

/** @param array{v: int|null} $arr */
function testIsNotNull(array $arr): int {
    if (!is_null($arr['v'])) {
        return $arr['v'] + 1;
    }
    return 0;
}
===expect===
