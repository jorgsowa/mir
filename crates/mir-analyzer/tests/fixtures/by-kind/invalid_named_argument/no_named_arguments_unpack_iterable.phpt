===description===
No named arguments unpack iterable
===file===
<?php
/**
 * @suppress UnusedParam
 * @no-named-arguments
 */
function foo(int $arg1, int $arg2): void {}

/** @var iterable<string, int> */
$test = ["arg1" => 1, "arg2" => 2];
foo(...$test);

===expect===
UnusedPsalmSuppress@6:0-6:0: Suppress annotation for 'UnusedParam' is never used
