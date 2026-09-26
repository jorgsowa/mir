===description===
A variable resolves to its narrowed type at that point.
===cursor===
symbol
===file===
<?php
function f(?int $n): void {
    if ($n !== null) {
        echo $<CURSOR>n;
    }
}
===expect===
kind: variable $n
type: int
