===description===
Assert not empty on bool
===ignore===
TODO
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
