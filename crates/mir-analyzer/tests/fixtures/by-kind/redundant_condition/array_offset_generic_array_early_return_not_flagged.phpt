===description===
L3, narrowing-divergence side: without the `possibly_absent_offset` fix,
`if ($row === null) { return; }` on a generic-array offset read was treated
as an impossible/dead branch (the stored type never literally includes
`null`), producing both a condition-level diagnostic and (via
`narrow_var_null`'s divergence marking) a dead-branch one on the `return;`
itself. Neither should fire once presence is correctly treated as unproven.
===config===
suppress=UnusedParam
===file===
<?php
/** @param array<string, array<string, int>> $matrix */
function test(array $matrix): int {
    $row = $matrix['a'];
    if ($row === null) {
        return 0;
    }
    return $row['b'] ?? 0;
}
===expect===
