===description===
reports deprecated function at top level
===file===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}

oldGreet('Alice');
===expect===
UnusedParam@3:18-3:30: Parameter $name is never used
DeprecatedCall@5:0-5:17: Call to deprecated function oldGreet: use newGreet() instead
