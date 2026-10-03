===description===
InvalidArrayAccess does NOT fire when the type includes an array variant.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var array<int>|null $x */
$x = null;
if ($x !== null) {
    $val = $x[0];
}

===expect===
