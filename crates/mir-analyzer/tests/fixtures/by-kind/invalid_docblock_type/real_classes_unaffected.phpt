===description===
Fully qualified classes and unqualified keywords remain valid docblock types.
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
