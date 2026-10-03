===description===
Arrays can never be loosely equal to floats in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $arr, float $f): void {
    if ($arr == $f) {}
//      ^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'array' and 'float' is always false — these types can never be loosely equal
}
===expect===
