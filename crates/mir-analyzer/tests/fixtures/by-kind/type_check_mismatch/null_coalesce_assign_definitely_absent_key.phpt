===description===
`$arr['a'] ??= $v` on a shape that provably lacks key 'a' behaves like the
existing undefined-variable case: the right-hand side always runs, so the
result is exactly `$v`'s type — not a union with the `mixed` a plain read of
a definitely-absent offset would otherwise produce.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $arr = [];
    $arr['a'] ??= 1;
    /** @mir-check $arr is array{'a': 1} */
    $_ = $arr;
}
===expect===
