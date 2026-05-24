===description===
mir-check inside anonymous callable body emits error
===file===
<?php
$fn = fn(int $x): int => (/** @mir-check $x is string */ $x * 2);
===expect===
TypeCheckMismatch@2:57: Type of $x is expected to be string, got int
