===description===
MissingClosureReturnType does NOT fire for arrow functions (fn() => expr) — their
return type is inferred from the expression body, so no annotation is required.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$fn = fn() => 1;
$fn2 = fn(int $x) => $x + 1;
