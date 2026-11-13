===description===
A variable assigned inside a property fetch's receiver or offset in the right operand of `&&`
is defined in the truthy branch.
===file===
<?php
final class E { public int $p = 0; }
function mk(): E { return new E(); }

/** @param list<E> $d */
function offset(array $d, int $n): int {
    if ($n > 0 && $d[$i = $n - 1]->p > 0) {
        return $i;
    }
    return 0;
}

function receiver(bool $a): ?E {
    if ($a && ($e = mk())->p === 1) {
        return $e;
    }
    return null;
}
