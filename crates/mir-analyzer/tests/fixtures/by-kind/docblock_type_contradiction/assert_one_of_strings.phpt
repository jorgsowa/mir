===description===
Assert one of strings
===config===
suppress=UnusedParam
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
DocblockTypeContradiction@9:9-9:19: Type '"a"|"b"' makes '$s === "c"' impossible — this can never hold
