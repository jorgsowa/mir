===description===
A by-reference captured variable resolves at the `use` clause.
===cursor===
symbol
===file===
<?php
function f(): void {
    $total = 0;
    $add = function (int $n) use (&$to<CURSOR>tal): void {
        $total += $n;
    };
    $add(1);
}
===expect===
kind: variable $total
type: int
