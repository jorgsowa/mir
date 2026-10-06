===description===
variable in type position param
===file===
<?php
/**
 * @param Foo|$invalid $x
// ^^^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: @param contains variable `$invalid` in type position
 */
function foo(mixed $x): void {}
//           ^^^^^^^^ UnusedParam: Parameter $x is never used
