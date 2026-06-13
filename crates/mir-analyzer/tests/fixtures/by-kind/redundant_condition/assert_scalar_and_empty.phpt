===description===
Assert scalar and empty
===config===
suppress=MixedArgument,UnusedParam
===file===
<?php
/**
 * @param mixed $value
 * @assert scalar $value
 * @assert !empty $value
 */
function assertScalarNotEmpty($value) : void {}

/** @param scalar $s */
function takesScalar($s) : void {}

/**
 * @param mixed $bar
 */
function foo($bar) : void {
    assertScalarNotEmpty($bar);
    takesScalar($bar);

    if ($bar) {}
}
===expect===
