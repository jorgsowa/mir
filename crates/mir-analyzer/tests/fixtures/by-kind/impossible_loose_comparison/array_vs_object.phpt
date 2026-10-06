===description===
Arrays can never be loosely equal to objects in PHP.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(array $arr, \stdClass $obj): void {
    if ($arr == $obj) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'array' and 'stdClass' is always false — these types can never be loosely equal
}
