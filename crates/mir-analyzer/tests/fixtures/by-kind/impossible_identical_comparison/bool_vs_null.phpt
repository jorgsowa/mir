===description===
A bool-typed variable can never be === to null.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(bool $b): void {
    if ($b === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'bool' and 'null' is always false — these types can never be identical
}
