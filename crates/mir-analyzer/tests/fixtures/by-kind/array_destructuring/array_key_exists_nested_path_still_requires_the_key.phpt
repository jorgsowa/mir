===description===
Negative counterpart: `array_key_exists()` on a nested path only narrows the
key it actually checks — an unrelated optional sibling key is still nullable.
===config===
suppress=MixedAssignment
===file===
<?php
/** @param array{a: array{b?: string, c?: string}} $arr */
function f(array $arr): void {
    if (array_key_exists('b', $arr['a'])) {
        $c = $arr['a']['c'];
        /** @mir-check $c is string|null */
        $_ = $c;
    }
}
===expect===
