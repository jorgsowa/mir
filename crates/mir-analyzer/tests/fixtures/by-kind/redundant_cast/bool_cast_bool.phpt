===description===
Redundant cast from bool to bool

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = true;
$y = (bool)$x;
//         ^^ RedundantCast: Casting 'true' to 'bool' is redundant

===expect===
