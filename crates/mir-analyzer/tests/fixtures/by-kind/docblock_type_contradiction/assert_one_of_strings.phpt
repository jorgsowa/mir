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
DocblockTypeContradiction@9:8-9:18: Type '"a"|"b"' makes '$s === "c"' impossible — this can never hold
ImpossibleIdenticalComparison@9:8-9:18: '===' between '"a"|"b"' and '"c"' is always false — these types can never be identical
RedundantCondition@9:8-9:18: Condition is always true/false for type 'bool'
