===description===
An arrow function's parameter resolves inside its body.
===cursor===
symbol
===file===
<?php
$f = fn(int $x): int => $<CURSOR>x * 2;
===expect===
kind: variable $x
type: int
