===file===
<?php
/**
 * @param Foo|$invalid $x
 */
function foo(mixed $x): void {}
===expect===
InvalidDocblock: Invalid docblock: @param contains variable `$invalid` in type position
UnusedParam: Parameter $x is never used
