===description===
A ternary whose type check always passes never reaches its else branch; negating the check flips the verdict.
===file===
<?php
function size(string $name): int {
    return is_string($name) ? strlen($name) : 0;
//         ^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always true, so the else branch is never reached
}

function negated(int $id): string {
    return !is_int($id) ? 'invalid' : 'valid';
//         ^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}
