===description===
When both @param (in-type) and @param-out (out-type) are declared, the out-type
takes effect for the variable after the call. The in-type is unaffected.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @param mixed $value
 * @param-out string $value
 */
function stringify(mixed &$value): void {
    $value = (string) $value;
}

$v = 42;
stringify($v);
/** @mir-check $v is string */
$_ = $v;
===expect===
