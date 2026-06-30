===description===
MixedArrayOffset does NOT fire after an explicit (int) cast — the cast produces a concrete int type
===config===
suppress=UnusedVariable
===file===
<?php
/** @var mixed $x */
$x = 0;
$arr = [10, 20, 30];
$val = $arr[(int) $x];
===expect===
