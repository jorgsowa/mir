===description===
Assert not empty on bool
===file===
<?php
/**
 * @param mixed $value
 * @assert !empty $value
 */
function assertNotEmpty($value) : void {}

function foo(bool $bar) : void {
    assertNotEmpty($bar);
    if ($bar) {}
}
===expect===
