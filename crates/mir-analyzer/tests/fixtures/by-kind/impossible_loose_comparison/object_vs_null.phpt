===description===
Objects can never be loosely equal to null in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(\stdClass $obj): void {
    if ($obj == null) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'null' is always false — these types can never be loosely equal
}
