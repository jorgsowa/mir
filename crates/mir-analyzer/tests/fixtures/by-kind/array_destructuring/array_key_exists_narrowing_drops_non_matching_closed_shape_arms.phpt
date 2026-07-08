===description===
array_key_exists('a', $arr) on a union of closed shapes must exclude the
arms that lack the key entirely, not spuriously add it as mixed — otherwise
a later access still widens to mixed instead of the surviving arm's real
value type. A single (non-union) closed shape lacking the key must still
fall back to the lenient add-as-mixed behavior, since a lone docblock shape
isn't proof the underlying array holds no other keys.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{type: string, a: int}|array{type: string, b: string} $arr
 */
function narrows_union(array $arr): void {
    if (array_key_exists('a', $arr)) {
        $val = $arr['a'];
        /** @mir-check $val is int */
        echo 1;
    }
}

/**
 * @param array{title: string} $arr
 */
function single_shape_stays_lenient(array $arr): void {
    if (array_key_exists('favicon', $arr)) {
        $val = $arr['favicon'];
        echo 1;
    }
}
===expect===
MixedAssignment@18:8-18:30: Variable $val is assigned a mixed type
