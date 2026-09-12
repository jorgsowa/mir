===description===
`is_string()` narrows a literal array-offset type.
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
