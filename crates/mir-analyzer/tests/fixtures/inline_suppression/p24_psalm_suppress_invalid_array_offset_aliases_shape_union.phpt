===description===
The alias also covers the heterogeneous-union path: a closed shape alongside a
generic array, read with a key the shape does not declare.
===file===
<?php
/** @param array{a: int}|array<string, string> $v */
function test(array $v): void {
    /** @psalm-suppress InvalidArrayOffset */
    $x = $v['missing'];
    /** @mir-check $x is mixed */
    echo $x;
}
===expect===
MixedAssignment@5:4-5:22: Variable $x is assigned a mixed type
