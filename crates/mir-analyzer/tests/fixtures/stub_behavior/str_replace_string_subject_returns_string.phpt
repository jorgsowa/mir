===description===
str_replace and str_ireplace with string subject return string, not string|array
===file===
<?php
$parts = ['rgb(100,200,150)', '200', '150'];
$r = (int) str_ireplace(['rgb(', 'rgba('], ['', ''], $parts[0]);
$g = (int) str_replace(')', '', $parts[2]);

$x = str_replace('a', 'b', 'hello');
/** @mir-check $x is string */
echo $x;

$y = str_ireplace('a', 'b', 'hello');
/** @mir-check $y is string */
echo $y;
===expect===
