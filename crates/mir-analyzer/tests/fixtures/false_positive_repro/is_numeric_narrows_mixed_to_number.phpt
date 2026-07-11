===description===
FP: is_numeric($x) narrows mixed/scalar to int|float|numeric-string in the
truthy branch, like the other is_*() type-guard functions do. Previously
TMixed/TScalar were kept unchanged, so arithmetic on $x downstream still
reported MixedAssignment/MixedReturnStatement even though is_numeric() had
already ruled out non-numeric values.
===file===
<?php
function f(mixed $x): int|float {
    if (is_numeric($x)) {
        return $x + 0;
    }
    return 0;
}
===expect===
