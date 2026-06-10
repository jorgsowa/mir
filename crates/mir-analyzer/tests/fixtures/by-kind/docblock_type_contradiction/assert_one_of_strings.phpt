===description===
Assert one of strings
===file===
<?php
/**
 * @assert "a"|"b" $s
 */
function foo(string $s) : void {}

function takesString(string $s) : void {
    foo($s);
    if ($s === "c") {}
}
===expect===
