===description===
Objects can never be loosely equal to arrays in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(\stdClass $obj, array $arr): void {
    if ($obj == $arr) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'array' is always false — these types can never be loosely equal
}
