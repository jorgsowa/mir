===description===
Built-in function with mixed wrong casing is detected.
===config===
suppress=UnusedVariable
===file===
<?php
$x = Array_Map(fn($v) => $v * 2, [1, 2, 3]);
//   ^^^^^^^^^ WrongCaseFunction: Function name 'Array_Map' has incorrect casing; use 'array_map'
===expect===
