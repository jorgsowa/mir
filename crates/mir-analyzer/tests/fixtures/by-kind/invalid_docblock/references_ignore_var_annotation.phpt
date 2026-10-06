===description===
References ignore var annotation
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = 1;
/** @var int */
$b = &$a;
