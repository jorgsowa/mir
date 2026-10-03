===description===
var_export with $return=true returns string, not string|null
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$exported = var_export(['key' => 'value'], true);
/** @mir-check $exported is string */
echo $exported;
===expect===
