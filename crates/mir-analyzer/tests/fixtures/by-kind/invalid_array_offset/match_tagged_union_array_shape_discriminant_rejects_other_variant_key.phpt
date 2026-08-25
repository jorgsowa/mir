===description===
After matching on a tagged-union array-shape discriminant, reading the other
variant's key inside an arm must still be rejected.
===config===
suppress=UnusedVariable
===file===
<?php
/** @param array{type: 'a', foo: int}|array{type: 'b', bar: string} $x */
function f(array $x): void {
    match ($x['type']) {
        'a' => $x['bar'],
        'b' => $x['bar'],
    };
}
===expect===
NonExistentArrayOffset@5:18-5:23: Array offset 'bar' does not exist
