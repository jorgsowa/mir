===description===
Built-in function with mixed wrong casing is detected.
===file===
<?php
Array_Map(fn($v) => $v * 2, [1, 2, 3]);
===expect===
WrongCaseFunction@2:1-2:10: Function name 'Array_Map' has incorrect casing; use 'array_map'
