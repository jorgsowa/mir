===description===
Redundant cast from int literal to int

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (int)3;
//        ^ RedundantCast: Casting '3' to 'int' is redundant
