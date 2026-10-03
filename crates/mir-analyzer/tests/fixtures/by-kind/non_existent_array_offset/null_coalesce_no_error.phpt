===description===
No NonExistentArrayOffset on the LHS of ??
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
$a = ["k" => 1];
$x = $a["missing"] ?? "default";
===expect===
