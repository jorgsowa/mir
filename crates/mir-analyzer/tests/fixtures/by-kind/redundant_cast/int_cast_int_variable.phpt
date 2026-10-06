===description===
Redundant cast from int variable to int

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = 3;
$y = (int)$x;
//        ^^ RedundantCast: Casting '3' to 'int' is redundant
