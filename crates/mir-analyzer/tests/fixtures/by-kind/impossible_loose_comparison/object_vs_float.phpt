===description===
Objects can never be loosely equal to floats in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(\stdClass $obj, float $f): void {
    if ($obj == $f) {}
//      ^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'float' is always false — these types can never be loosely equal
}
===expect===
