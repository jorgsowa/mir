===description===
G11: the whole `is_*`/`ctype_*` type-check narrowing family never applied to
array-offset expressions, same gap as `instanceof`. `is_string($arr['v'])`
on a literal-keyed shape access must narrow the key's own stored type, so a
later read only sees `string`.
===file===
<?php
/** @param array{v: string|int} $arr */
function test(array $arr): string {
    if (is_string($arr['v'])) {
        return strtoupper($arr['v']);
    }
    return '';
}
===expect===
