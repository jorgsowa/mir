===description===
Backslash-qualified REAL classes are valid fully qualified names and must
not be flagged — nor must un-backslashed keywords. The info-level
UndefinedDocblockClass (hidden without `--show-info`) still fires for the
unresolvable `\Foo\Bar`.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param \Closure $a
 * @param \Exception $e
 * @param Closure $b
 * @param int $c
 * @return \Foo\Bar
 */
function f($a, $e, $b, $c): string {
    return "x";
}
===expect===
UndefinedDocblockClass@9:9-9:10: Docblock type 'Foo\Bar' does not exist
