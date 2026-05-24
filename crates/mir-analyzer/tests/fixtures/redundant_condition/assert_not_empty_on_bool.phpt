===description===
assertNotEmptyOnBool
===file===
<?php
/**
 * @param mixed $value
 * @psalm-assert !empty $value
 */
function assertNotEmpty($value) : void {}

function foo(bool $bar) : void {
    assertNotEmpty($bar);
    if ($bar) {}
}
===expect===
RedundantConditionGivenDocblockType
===ignore===
TODO
