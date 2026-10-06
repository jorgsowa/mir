===description===
Using literal float value as array offset - silently truncated to int

===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$arr = [];
$val = $arr[3.7];
//          ^^^ ImplicitFloatToIntCast: Implicit cast from 3.7 to int truncates the fractional part
//          ^^^ NonExistentArrayOffset: Array offset '3' does not exist
