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
//       ^ UndefinedDocblockClass: Docblock type 'Foo\Bar' does not exist
    return "x";
}
===expect===
