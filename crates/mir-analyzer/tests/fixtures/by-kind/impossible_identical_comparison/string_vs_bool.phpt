===description===
A string-typed variable can never be === to a boolean.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(string $s): void {
    if ($s === false) {}
//      ^^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'false' is always false — these types can never be identical
    if ($s === true) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'true' is always false — these types can never be identical
}
===expect===
