===description===
MismatchingDocblockParamType fires when a docblock @param contradicts the
native hint; narrowing or matching docblock params stay silent.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param int $a
 * @param non-empty-string $b
 * @param string $c
 */
function f(string $a, string $b, string $c): void {}
//                ^^ MismatchingDocblockParamType: Docblock type 'int' for $a does not match inferred 'string'
===expect===
