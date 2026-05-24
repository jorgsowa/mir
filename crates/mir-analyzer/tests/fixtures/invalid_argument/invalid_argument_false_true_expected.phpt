===description===
invalidArgumentFalseTrueExpected
===file===
<?php
/**
 * @param true|string $arg
 * @return void
 */
function foo($arg) {}

foo(false);
===expect===
UnusedParam@6:13: Parameter $arg is never used
InvalidArgument@8:4: Argument $arg of foo() expects 'true|string', got 'false'
