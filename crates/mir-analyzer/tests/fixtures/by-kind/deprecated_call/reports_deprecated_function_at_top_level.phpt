===description===
reports deprecated function at top level
===file===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}
//                ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used

oldGreet('Alice');
//<^^^^^^^^^^^^^^^^^ DeprecatedCall: Call to deprecated function oldGreet: use newGreet() instead
===expect===
