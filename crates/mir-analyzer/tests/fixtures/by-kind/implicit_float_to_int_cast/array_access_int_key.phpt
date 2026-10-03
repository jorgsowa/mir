===description===
Using int value as array offset - should not emit ImplicitFloatToIntCast

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
$val = $arr[3];
//          ^ NonExistentArrayOffset: Array offset '3' does not exist

===expect===
