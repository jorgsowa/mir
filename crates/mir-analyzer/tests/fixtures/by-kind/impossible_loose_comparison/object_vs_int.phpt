===description===
Objects can never be loosely equal to integers in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(\stdClass $obj, int $n): void {
    if ($obj == $n) {}
//      ^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'int' is always false — these types can never be loosely equal
}
===expect===
