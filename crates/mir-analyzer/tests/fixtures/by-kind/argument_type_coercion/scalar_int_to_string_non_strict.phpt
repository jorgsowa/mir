===description===
In non-strict PHP, passing int/float to a string-typed parameter is a benign coercion.
Should emit ArgumentTypeCoercion (Info), not InvalidArgument (Error).
===config===
suppress=UnusedParam
===file===
<?php
/** @param string $s */
function takes_string(string $s): void { echo $s; }

takes_string(1);
takes_string(42);
takes_string(3.14);
===expect===
ArgumentTypeCoercion@5:13-5:14: Argument $s of takes_string() expects 'string', got '1' — coercion may fail at runtime
ArgumentTypeCoercion@6:13-6:15: Argument $s of takes_string() expects 'string', got '42' — coercion may fail at runtime
ArgumentTypeCoercion@7:13-7:17: Argument $s of takes_string() expects 'string', got '3.14' — coercion may fail at runtime
