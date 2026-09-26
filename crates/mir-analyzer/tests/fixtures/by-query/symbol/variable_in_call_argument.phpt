===description===
A variable passed as an argument resolves to the variable, not the call.
===cursor===
symbol
===file===
<?php
function f(string $s): int {
    return strlen($<CURSOR>s);
}
===expect===
kind: variable $s
type: string
