===description===
Invalid array offset
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = ["a"];
$y = $x["b"];
//      ^^^ NonExistentArrayOffset: Array offset 'b' does not exist
===expect===
