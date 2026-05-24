===description===
assertScalarAndEmpty
===file===
<?php
/**
 * @param mixed $value
 * @psalm-assert scalar $value
 * @psalm-assert !empty $value
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
RedundantConditionGivenDocblockType
===ignore===
TODO
