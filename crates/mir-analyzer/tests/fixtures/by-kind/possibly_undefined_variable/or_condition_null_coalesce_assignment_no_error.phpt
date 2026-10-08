===description===
When `A || B` is false both operands ran, including an assignment nested in an
index expression under `??` in B, so a guard that returns on true leaves it defined.
===file===
<?php
function guarded(string $s, int $p): int {
    if ($s[$p] !== '{' || ($s[$e = strlen($s) - $p] ?? null) !== '}') {
        return 0;
    }
    /** @mir-check $e is int */
    return $e;
}

function coalesceRightNotPromoted(?string $s): int {
    if ($s === null || ($s ?? ($e = 1)) === 'x') {
        return 0;
    }
    return $e;
//         ^^ PossiblyUndefinedVariable: Variable $e might not be defined
}
