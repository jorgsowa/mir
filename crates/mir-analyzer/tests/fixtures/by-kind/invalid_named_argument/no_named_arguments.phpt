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
UnusedPsalmSuppress@6:0-6:0: Suppress annotation for 'UnusedParam' is never used
InvalidNamedArguments@8:5-8:12: foo() does not accept named arguments
InvalidNamedArguments@8:14-8:21: foo() does not accept named arguments
