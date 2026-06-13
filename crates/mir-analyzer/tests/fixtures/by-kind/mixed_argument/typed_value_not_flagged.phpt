===description===
MixedArgument does NOT fire when the argument has a concrete (non-mixed) type.
===file===
<?php
function foo(int $a): void {}
foo(42);

===expect===
