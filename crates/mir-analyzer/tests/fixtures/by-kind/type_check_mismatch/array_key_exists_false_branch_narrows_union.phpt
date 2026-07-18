===description===
`!array_key_exists('a', $arr)` on a union of closed shapes must exclude the
arms that guarantee the key's presence — the false-branch counterpart of
array_key_exists_narrowing_drops_non_matching_closed_shape_arms.phpt, which
does the same for the true branch in the opposite direction. A single
(non-union) closed shape stays untouched even when it declares the key
mandatory, mirroring the true-branch helper's same lone-shape leniency.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @param array{type: string, a: int}|array{type: string, b: string} $arr
 */
function false_branch_excludes_shape_with_mandatory_key(array $arr): void {
    if (!array_key_exists('a', $arr)) {
        /** @mir-check $arr is array{type: string, b: string} */
        $_ = $arr;
    }
}

/**
 * @param array{title: string} $arr
 */
function single_shape_false_branch_stays_lenient(array $arr): void {
    if (!array_key_exists('title', $arr)) {
        /** @mir-check $arr is array{title: string} */
        $_ = $arr;
    }
}

/**
 * @param array{a?: int, type: string}|array{type: string, b: string} $arr
 */
function optional_key_shape_survives_false_branch(array $arr): void {
    // 'a' is optional in the first arm, so it's still consistent with the
    // key's absence — that arm must not be excluded.
    if (!array_key_exists('a', $arr)) {
        /** @mir-check $arr is array{a?: int, type: string}|array{type: string, b: string} */
        $_ = $arr;
    }
}
===expect===
