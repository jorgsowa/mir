===description===
variable in type position param
===file===
<?php
/**
 * @param Foo|$invalid $x
 */
function foo(mixed $x): void {}
//           ^^^^^^^^ UnusedParam: Parameter $x is never used
===expect===
InvalidDocblock@3:3-3:25: Invalid docblock: @param contains variable `$invalid` in type position
