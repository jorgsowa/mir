===description===
Invalid explicit cast from array to int

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (int)[];
//        ^^ InvalidCast: Cannot cast 'array{}' to 'int'

===expect===
