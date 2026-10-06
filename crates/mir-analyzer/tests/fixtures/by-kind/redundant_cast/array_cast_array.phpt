===description===
Redundant cast from array to array

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = [];
$y = (array)$x;
//          ^^ RedundantCast: Casting 'array{}' to 'array' is redundant
