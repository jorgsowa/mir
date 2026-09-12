===file===
<?php
/**
 * B8: interpolated strings / heredocs with a non-empty literal part are
 * guaranteed non-empty no matter how the embedded expressions coerce to
 * string, so the whole string must type as `non-empty-string` (same rule
 * as `.` concatenation — `is_non_empty_when_concat` in binary.rs).
 *
 * The trigger shape is a `false|non-empty-string` docblock return union:
 * before the fix the interpolation degraded to `string`, which the
 * return-type check rejects against the union (InvalidReturnType), even
 * though a non-empty literal part makes the empty-string case impossible.
 *
 * The `no_literal` control has no literal part — the embedded expression
 * can still be "", so the result stays `string` and MUST keep failing the
 * same union.
 */

/**
 * @param string $x
 *
 * @return false|non-empty-string
 */
function interpolated_prefix(string $x): false|string {
    if ($x === '') {
        return false;
    }
    return "prefix{$x}";
}

/**
 * @param string $x
 *
 * @return false|non-empty-string
 */
function interpolated_middle(string $x): false|string {
    if ($x === '') {
        return false;
    }
    return "pre{$x}post";
}

/**
 * @param string $x
 *
 * @return false|non-empty-string
 */
function heredoc_prefix(string $x): false|string {
    if ($x === '') {
        return false;
    }
    return <<<EOT
prefix{$x}
EOT;
}

// Inference pin: a non-empty literal part refines the whole
// interpolation to `non-empty-string`, exactly like `.` concatenation.
function interpolation_result(string $x): string {
    $s = "prefix{$x}";
    /** @mir-check $s is non-empty-string */
    return $s;
}

/**
 * @param string $a
 * @param string $b
 *
 * @return false|non-empty-string
 */
function no_literal(string $a, string $b): false|string {
    if ($a === '') {
        return false;
    }
    return "{$b}";
}
===expect===
InvalidReturnType@74:4-74:18: Return type 'string' is not compatible with declared 'false|non-empty-string'
