===description===
A non-nullable string-typed variable can never be === to null.
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
    if ($s === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
}
===expect===
