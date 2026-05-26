===description===
SKIPPED-preventUnpackingPossiblyArray
===file===
<?php
function foo(int $arg1, int $arg2): void {}

/** @var array<int, int>|object */
$test = [1, 2];
foo(...$test);

===expect===
PossiblyInvalidArgument
===ignore===
TODO
