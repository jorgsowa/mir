===description===
D4: a docblock-only `@return` (no native return-type hint) on an arrow
function is also checked, mirroring the same `@return` fallback already
used for a regular closure.
===config===
suppress=UnusedVariable
===file===
<?php
$f =
    /** @return int */
    fn() => 'not an int';
===expect===
InvalidReturnType@4:12-4:24: Return type '"not an int"' is not compatible with declared 'int'
