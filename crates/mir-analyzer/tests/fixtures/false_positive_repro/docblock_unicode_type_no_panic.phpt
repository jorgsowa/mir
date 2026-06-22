===description===
A1: docblock type names that lowercase to a different byte length (e.g., Ⱥ, Ⱦ)
must not panic with a char-boundary slice. The analyzer must process the file
without crashing and emit at most a parse-warning, never an ICE.
===config===
suppress=UndefinedDocblockClass,UnusedParam
php_version=8.2
===file===
<?php

/**
 * @param Ⱥrray<int, string> $x
 */
function foo($x): void {}
===expect===
