===description===
Prevent unpacking possibly iterable
===config===
suppress=MixedArgument,UnusedParam
===file===
<?php
function foo(int $arg1, int $arg2): void {}

/** @var iterable<int, int>|object */
$test = [1, 2];
foo(...$test);

===expect===
