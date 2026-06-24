===description===
@param-out on a non-by-reference parameter has no effect — the variable type
after the call should remain unchanged. (Declare the param &$val to see it work.)
===config===
suppress=UnusedVariable,UnusedFunction,MixedAssignment,UnusedParam
===file===
<?php
/**
 * @param-out string $val
 */
function noEffect(mixed $val): void {
    // not by-ref: $val is a copy
}

$x = 42;
noEffect($x);
/** @mir-check $x is int */
$_ = $x;
===expect===
