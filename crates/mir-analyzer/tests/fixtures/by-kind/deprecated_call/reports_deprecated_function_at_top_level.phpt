===description===
reports deprecated function at top level
===file===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}

oldGreet('Alice');
===expect===
UnusedParam@3:19-3:31: Parameter $name is never used
DeprecatedCall@5:1-5:18: Call to deprecated function oldGreet: use newGreet() instead
