===description===
reports deprecated function at top level
===file===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}

oldGreet('Alice');
===expect===
UnusedParam@3:18: Parameter $name is never used
DeprecatedCall@5:0: Call to deprecated function oldGreet: use newGreet() instead
