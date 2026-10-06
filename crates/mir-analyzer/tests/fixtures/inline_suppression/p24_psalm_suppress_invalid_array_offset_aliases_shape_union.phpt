===description===
The alias also covers the heterogeneous-union path: a closed shape alongside a
non-array member, read with a key the shape does not declare.
===file===
<?php
function test(bool $c): void {
    $v = $c ? ['a' => 1] : 'text';
    /** @psalm-suppress InvalidArrayOffset */
    $x = $v['missing'];
//  ^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $x is assigned a mixed type
    /** @mir-check $x is mixed */
    echo $x;
}
