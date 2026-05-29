===description===
str_replace and str_ireplace with array subject return array, not string
===file===
<?php
$subjects = ['hello', 'world'];
$x = str_replace('l', 'r', $subjects);
/** @mir-check $x is array<int, string> */
$_ = $x;

$y = str_ireplace('L', 'r', $subjects);
/** @mir-check $y is array<int, string> */
$_ = $y;
===expect===
