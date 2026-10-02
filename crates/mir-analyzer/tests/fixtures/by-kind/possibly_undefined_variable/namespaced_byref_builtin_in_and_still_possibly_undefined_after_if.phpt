===description===
Outside the true branch of `&&` the out param is still possibly undefined,
because the right operand may have been short-circuited.
===file===
<?php
namespace App;

function f(?string $p): ?int {
    if ($p !== null && preg_match('/(\d+)/', $p, $m) === 1) {
        return 1;
    }
    return (int) $m[1];
}
===expect===
PossiblyUndefinedVariable@8:17-8:19: Variable $m might not be defined
