===description===
checkCallableTypeString
===file===
<?php
/**
 * @param callable(int,int):int $_p
 */
function f(callable $_p): void {}

f("strcmp");
===expect===
InvalidScalarArgument
===ignore===
TODO
