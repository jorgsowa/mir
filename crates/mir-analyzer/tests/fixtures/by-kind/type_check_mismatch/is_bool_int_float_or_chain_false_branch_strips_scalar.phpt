===description===
M31: `is_bool($x)||is_int($x)||is_float($x)`'s else-branch over a value
previously narrowed to include the opaque `scalar` atom (e.g. via a
`!is_scalar($x)` false-branch merge) never stripped the leftover `TScalar` —
unlike `is_scalar()`'s own false-branch, which already excludes it. Ruling
out bool/int/float from a value that's provably one of the four scalar
kinds must also rule out the still-unresolved `scalar` atom itself.
===file===
<?php
function test(mixed $value): void {
    if (is_string($value)) {
        // $value: string
    } elseif (is_scalar($value)) {
        // $value: scalar
    } else {
        return;
    }
    if (is_bool($value) || is_int($value) || is_float($value)) {
        $value = 'x';
    } else {
        /** @mir-check $value is string */
        echo $value;
    }
}
===expect===
