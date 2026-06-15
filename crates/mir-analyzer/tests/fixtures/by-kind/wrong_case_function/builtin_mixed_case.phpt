===description===
Built-in function with mixed wrong casing is detected.
===config===
suppress=UnusedVariable
===file===
<?php
$x = Array_Map(fn($v) => $v * 2, [1, 2, 3]);
===expect===
WrongCaseFunction@2:5-2:14: Function name 'Array_Map' has incorrect casing; use 'array_map'
