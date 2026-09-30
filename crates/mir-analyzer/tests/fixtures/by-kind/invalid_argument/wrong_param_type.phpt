===description===
Wrong param type
===file===
<?php
$take_string = function(string $s): string { return $s; };
$take_string(42);
//           ^^ ArgumentTypeCoercion: Argument $s of {closure}() expects 'string', got '42' — coercion may fail at runtime
===expect===
