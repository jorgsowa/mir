===description===
Under strict_types=1, passing int to a string parameter is a genuine type error (no PHP coercion).
Should emit InvalidArgument (Error), not ArgumentTypeCoercion.
===config===
suppress=UnusedParam
===file===
<?php
declare(strict_types=1);

/** @param string $s */
function takes_string(string $s): void { echo $s; }

takes_string(1);
===expect===
InvalidArgument@7:13-7:14: Argument $s of takes_string() expects 'string', got '1'
