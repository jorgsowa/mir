===description===
Invalid explicit cast from array to string

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (string)[];
//           ^^ InvalidCast: Cannot cast 'array{}' to 'string'
