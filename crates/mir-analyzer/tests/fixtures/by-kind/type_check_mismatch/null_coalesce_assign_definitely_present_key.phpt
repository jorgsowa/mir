===description===
`$arr['a'] ??= $v` on a shape that provably already has a non-optional,
non-null 'a' never runs its right-hand side — the result must stay exactly
the existing value, not a union with the never-assigned right-hand side.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $arr = ['a' => 5];
    $arr['a'] ??= 99;
    /** @mir-check $arr is array{'a': 5} */
    $_ = $arr;
}
===expect===
