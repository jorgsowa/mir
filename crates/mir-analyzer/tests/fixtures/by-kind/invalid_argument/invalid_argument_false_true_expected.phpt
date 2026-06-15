===description===
Invalid argument false true expected
===file===
<?php
/**
 * @param true|string $arg
 * @return void
 */
function foo($arg) {}

foo(false);
===expect===
UnusedParam@6:13-6:17: Parameter $arg is never used
InvalidArgument@8:4-8:9: Argument $arg of foo() expects 'true|string', got 'false'
