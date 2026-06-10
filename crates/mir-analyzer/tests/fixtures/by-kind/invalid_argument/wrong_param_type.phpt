===description===
Wrong param type
===file===
<?php
$take_string = function(string $s): string { return $s; };
$take_string(42);
===expect===
InvalidArgument@3:14-3:16: Argument $s of {closure}() expects 'string', got '42'
