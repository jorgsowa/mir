===description===
Assert not empty on bool
===config===
suppress=UnusedParam
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
