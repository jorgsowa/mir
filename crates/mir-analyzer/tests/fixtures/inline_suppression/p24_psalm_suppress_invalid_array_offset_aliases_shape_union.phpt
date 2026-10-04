===description===
The alias also covers the heterogeneous-union path: a closed shape alongside a
non-array member, read with a key the shape does not declare.
===file===
<?php
function test(bool $c): void {
    $v = $c ? ['a' => 1] : 'text';
    /** @psalm-suppress InvalidArrayOffset */
    $x = $v['missing'];
    /** @mir-check $x is mixed */
    echo $x;
}
===expect===
MixedAssignment@5:4-5:22: Variable $x is assigned a mixed type
