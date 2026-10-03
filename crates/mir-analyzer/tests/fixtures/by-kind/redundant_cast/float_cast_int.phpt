===description===
Widening cast from int to float - should not be redundant or error

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = 3;
$y = (float)$x;

===expect===
