===description===
Objects are always truthy in PHP, so an object can never be loosely equal to false.
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
    if ($obj == false) {}
//      ^^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'stdClass' and 'false' is always false — these types can never be loosely equal
}
