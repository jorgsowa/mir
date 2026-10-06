===description===
Invalid argument false true expected
===file===
<?php
/**
 * @param true|string $arg
 * @return void
 */
function foo($arg) {}
//           ^^^^ UnusedParam: Parameter $arg is never used

foo(false);
//  ^^^^^ InvalidArgument: Argument $arg of foo() expects 'true|string', got 'false'
