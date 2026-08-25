===description===
Matching on a tagged-union array-shape discriminant should reuse the literal
union inferred for that offset, so covering both tag literals is exhaustive
without a default arm.
===file===
<?php
/** @param array{type: 'a', foo: int}|array{type: 'b', bar: string} $x */
function label(array $x): string {
    return match ($x['type']) {
        'a' => 'left',
        'b' => 'right',
    };
}
===expect===
