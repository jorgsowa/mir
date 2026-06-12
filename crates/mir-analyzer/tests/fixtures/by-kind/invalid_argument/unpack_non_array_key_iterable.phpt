===description===
Unpack non array key iterable
===file===
<?php
/** @suppress UnusedParam */
function foo(string ...$args): void {}

/** @var Iterator<float, string> */
$test = null;
foo(...$test);

===expect===
UnusedPsalmSuppress@3:0-3:0: Suppress annotation for 'UnusedParam' is never used
