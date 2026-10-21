===description===
A static variable resolves at its declaration.
===cursor===
symbol
===file===
<?php
function counter(): int {
    static $cou<CURSOR>nt = 0;
    $count++;
    return $count;
}
===expect===
kind: variable $count
type: 0
