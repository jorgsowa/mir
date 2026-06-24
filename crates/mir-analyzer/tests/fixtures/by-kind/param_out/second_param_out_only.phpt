===description===
Only the second parameter has @param-out; the first param is positional and not
a byref — verifies correct param-index alignment in write-back.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @param-out string $out
 */
function prepare(int $x, mixed &$out): void {
    $out = "result:" . $x;
}

prepare(7, $s);
/** @mir-check $s is string */
$_ = $s;
===expect===
