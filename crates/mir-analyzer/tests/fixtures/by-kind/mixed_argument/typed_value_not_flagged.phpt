===description===
MixedArgument does NOT fire when the argument has a concrete (non-mixed) type.
===config===
suppress=UnusedParam
===file===
<?php
function foo(int $a): void {}
foo(42);

===expect===
