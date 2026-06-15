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
InvalidNamedArguments@8:4-8:11: foo() does not accept named arguments
InvalidNamedArguments@8:13-8:20: foo() does not accept named arguments
