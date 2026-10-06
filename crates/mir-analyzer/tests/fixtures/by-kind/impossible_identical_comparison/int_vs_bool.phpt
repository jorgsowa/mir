===description===
An integer-typed variable can never be === to a boolean.
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
    if ($x === true) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and 'true' is always false — these types can never be identical
    if ($x === false) {}
//      ^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'int' and 'false' is always false — these types can never be identical
}
