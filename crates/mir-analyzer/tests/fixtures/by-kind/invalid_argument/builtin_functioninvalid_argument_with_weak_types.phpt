===description===
Builtin functioninvalid argument with weak types
===config===
suppress=UnusedVariable
===file===
<?php
$s = substr(5, 4);
===expect===
ArgumentTypeCoercion@2:12-2:13: Argument $string of substr() expects 'string', got '5' — coercion may fail at runtime
