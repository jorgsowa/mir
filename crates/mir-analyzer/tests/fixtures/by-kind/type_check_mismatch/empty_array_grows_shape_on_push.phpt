===description===
A single `[]` push onto a proven-empty `list<T>` (straight-line code, not a
loop) grows the closed `array{}` shape by one property instead of collapsing
straight to `list<int>` — the same shape-preserving precision an array
literal `[1]` itself would type as.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param list<int> $arr */
function test(array $arr): void {
    if ($arr === []) {
        $arr[] = 1;
        /** @mir-check $arr is array{0: 1} */
        $_ = $arr;
    }
}
===expect===
