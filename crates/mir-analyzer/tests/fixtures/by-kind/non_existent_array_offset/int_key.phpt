===description===
Accessing a non-existent int key in a list array
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
