===description===
MixedArrayAccess does NOT fire when the array has a concrete element type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var array<int, string> $arr */
$arr = [];
$val = $arr[0];
