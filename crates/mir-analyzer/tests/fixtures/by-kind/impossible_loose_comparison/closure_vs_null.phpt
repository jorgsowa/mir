===description===
Closures are objects and can never be loosely equal to null.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(\Closure $fn): void {
    if ($fn == null) {}
//      ^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'Closure' and 'null' is always false — these types can never be loosely equal
}
