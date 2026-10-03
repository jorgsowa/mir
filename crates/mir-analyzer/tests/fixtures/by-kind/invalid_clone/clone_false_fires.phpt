===description===
InvalidClone fires when cloning a false literal (bool subtype).
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = false;
clone $x;
//<^^^^^^^^ InvalidClone: cannot clone non-object false
===expect===
