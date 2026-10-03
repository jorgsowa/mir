===description===
Assert one of strings
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @assert "a"|"b" $s
 */
function foo(string $s) : void {}

function takesString(string $s) : void {
    foo($s);
    if ($s === "c") {}
//      ^^^^^^^^^^ DocblockTypeContradiction: Type '"a"|"b"' makes '$s === "c"' impossible — this can never hold
//      ^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between '"a"|"b"' and '"c"' is always false — these types can never be identical
}
===expect===
