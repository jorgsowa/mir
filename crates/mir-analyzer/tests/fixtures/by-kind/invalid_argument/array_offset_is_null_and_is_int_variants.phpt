===description===
G11, more of the `type_fn_narrowed` family on array offsets: `is_int()`'s
true branch and `is_null()`'s false branch (which proves non-null the same
way a plain-variable/property receiver's false branch already does).
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
