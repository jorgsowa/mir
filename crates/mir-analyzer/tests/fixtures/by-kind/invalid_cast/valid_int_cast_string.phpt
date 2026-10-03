===description===
Valid cast from string to int - string is implicitly converted to int, should not emit InvalidCast

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (int)"42";

===expect===
