===description===
The != operator is also checked: object != null is always true.
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
    if ($obj != null) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '!=' between 'stdClass' and 'null' is always true — these types can never be loosely equal
}
===expect===
