===description===
Redundant cast from float literal to float

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (float)3.0;
//          ^^^ RedundantCast: Casting '3' to 'float' is redundant

===expect===
