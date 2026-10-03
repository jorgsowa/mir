===description===
Simple reference assignment ($b = &$a) does not fire UnsupportedReferenceUsage.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = "hello";
$b = &$a;

===expect===
