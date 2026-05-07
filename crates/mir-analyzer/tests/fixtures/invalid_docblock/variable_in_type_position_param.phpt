===description===
variable in type position param
===file===
<?php
/**
 * @param Foo|$invalid $x
 */
function foo(mixed $x): void {}
===expect===
InvalidDocblock@2:0: Invalid docblock: @param contains variable `$invalid` in type position
UnusedParam@5:13: Parameter $x is never used
