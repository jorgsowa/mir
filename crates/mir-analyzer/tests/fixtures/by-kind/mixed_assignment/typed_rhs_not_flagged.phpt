===description===
MixedAssignment does NOT fire when the right-hand side has a concrete type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = 42;
$b = $a;
