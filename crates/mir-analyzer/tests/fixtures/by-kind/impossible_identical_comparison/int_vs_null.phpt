===description===
An integer-typed variable can never be === to null.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(int $x): void {
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and 'null' is always false — these types can never be identical
}
===expect===
