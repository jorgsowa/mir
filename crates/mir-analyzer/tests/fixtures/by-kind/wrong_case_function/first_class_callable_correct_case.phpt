===description===
Correct case in first-class callable syntax is not reported.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function myFunc(int $x): int { return $x; }

$fn = myFunc(...);
===expect===
