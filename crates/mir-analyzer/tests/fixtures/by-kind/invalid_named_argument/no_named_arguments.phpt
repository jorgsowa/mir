===description===
No named arguments
===file===
<?php
/**
 * @suppress UnusedParam
 * @no-named-arguments
 */
function foo(int $arg1, int $arg2): void {}

foo(arg2: 0, arg1: 1);

===expect===
