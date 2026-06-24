===description===
@param-out declares the type written back to a by-ref argument variable after
the call. After calling such a function, the variable has the out-type, not the
in-type or mixed.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @param-out string $result
 */
function fill(mixed &$result): void {
    $result = "hello";
}

$x = null;
fill($x);
/** @mir-check $x is string */
$_ = $x;
===expect===
