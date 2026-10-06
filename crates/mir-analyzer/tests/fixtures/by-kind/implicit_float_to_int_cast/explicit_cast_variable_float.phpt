===description===
Explicit cast from float to int - user explicitly requests the cast, so no issue

===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = 3.7;
$y = (int)$x;
