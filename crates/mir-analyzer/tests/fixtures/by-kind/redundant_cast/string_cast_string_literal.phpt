===description===
Redundant cast from string literal to string

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = (string)"hello";
//           ^^^^^^^ RedundantCast: Casting '"hello"' to 'string' is redundant

===expect===
