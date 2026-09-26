===description===
A cursor on a variable's `$` sigil resolves the variable.
===cursor===
symbol
===file===
<?php
function f(int $n): void {
    echo <CURSOR>$n;
}
===expect===
kind: variable $n
type: int
