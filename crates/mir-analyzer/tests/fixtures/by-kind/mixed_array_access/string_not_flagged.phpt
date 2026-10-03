===description===
MixedArrayAccess does NOT fire for string offset access — a string is not mixed.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var string $str */
$str = "";
$ch = $str[0];
===expect===
