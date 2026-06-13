===description===
MissingParamType fires per untyped top-level function parameter; native hints
and docblock @param types both satisfy it.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param string $b
 */
function f($a, $b, int $c): void {}
===expect===
MissingParamType@5:12-5:14: Parameter $a of f() has no type annotation
