===description===
Matching on a tagged-union array-shape discriminant must narrow the original
shape union inside each arm, so variant-specific offset reads stay known and
the match result preserves both arm-local value types.
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @template T
 * @param T $x
 * @return T
 */
function id($x) {
    return $x;
}

/** @param array{type: 'a', foo: int}|array{type: 'b', bar: string} $x */
function f(array $x): void {
    $r = match ($x['type']) {
        'a' => id($x['foo']),
        'b' => id($x['bar']),
    };

    /** @mir-check $r is int|string */
    $_ = $r;
}
===expect===
