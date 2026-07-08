===description===
Reading an optional shape key (`array{b?: string}`) must widen the result
type with null — an optional key may genuinely be absent at runtime, so
treating it as always-present is unsound.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{a: int, b?: string} $arr
 */
function test(array $arr): void {
    $b = $arr['b'];
    /** @mir-check $b is string|null */
    echo 1;
}
===expect===
