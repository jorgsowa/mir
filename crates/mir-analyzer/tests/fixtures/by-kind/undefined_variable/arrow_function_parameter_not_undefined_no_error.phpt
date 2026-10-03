===description===
arrow function parameter not undefined no error
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$fn = fn(int $n): int => $n * 2;
===expect===
