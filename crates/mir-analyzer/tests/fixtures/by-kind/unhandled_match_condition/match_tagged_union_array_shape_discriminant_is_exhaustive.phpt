===description===
Matching every tagged-union discriminant value is exhaustive.
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
