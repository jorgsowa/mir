===description===
`foreach ($arr as [$a, $b])` / `foreach ($arr as ['k' => $v])` must resolve
each destructured variable's type from the iterated element type instead of
hard-setting every one to `mixed` — analyze_foreach_stmt's else branch (for
a non-plain-variable `fe.value`) never used the correctly-inferred
`value_ty`, unlike a plain `list($a, $b) = $arr` assignment which already
resolves per-element types via assign_to_target.
===file===
<?php

function tuples(): void {
    $pairs = [[1, 'a'], [2, 'b'], [3, 'c']];
    foreach ($pairs as [$num, $letter]) {
        /** @mir-check $num is int */
        echo $num;
        /** @mir-check $letter is string */
        echo $letter;
    }
}

function keyedShapes(): void {
    $items = [['x' => 1, 'y' => 'a'], ['x' => 2, 'y' => 'b']];
    foreach ($items as ['x' => $x, 'y' => $y]) {
        /** @mir-check $x is int */
        echo $x;
        /** @mir-check $y is string */
        echo $y;
    }
}
===expect===
