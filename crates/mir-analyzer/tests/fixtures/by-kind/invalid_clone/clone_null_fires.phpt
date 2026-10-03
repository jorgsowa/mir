===description===
InvalidClone fires when cloning a null literal.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$x = null;
clone $x;
//<^^^^^^^^ InvalidClone: cannot clone non-object null
===expect===
