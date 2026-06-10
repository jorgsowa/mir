===description===
Check callable type string
===file===
<?php
/**
 * @param callable(int,int):int $_p
 */
function f(callable $_p): void {}

f("strcmp");
===expect===
