===description===
Using float value as array key in array construction - silently truncated to int

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$arr = [1.5 => "value"];
//      ^^^ ImplicitFloatToIntCast: Implicit cast from 1.5 to int truncates the fractional part

===expect===
