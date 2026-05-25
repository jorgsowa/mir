===description===
assertOneOfStrings
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
DocblockTypeContradiction
===ignore===
TODO
