===description===
Possibly invalid array offset with int
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = rand(0, 5) > 2 ? ["a" => 5] : "hello";
$y = $x[0];
//      ^ NonExistentArrayOffset: Array offset '0' does not exist
===expect===
