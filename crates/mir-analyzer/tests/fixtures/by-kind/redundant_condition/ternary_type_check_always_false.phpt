===description===
A ternary whose type check can never pass is reported like the same check in an `if`.
===file===
<?php
function label(int $count): string {
    return is_string($count) ? 'text' : 'number';
//         ^^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}

function describe(float $ratio): string {
    return is_array($ratio) ? 'list' : 'scalar';
//         ^^^^^^^^^^^^^^^^ RedundantCondition: Condition is always false, so the then branch is never reached
}
