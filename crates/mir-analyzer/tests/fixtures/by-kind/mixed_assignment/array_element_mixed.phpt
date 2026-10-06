===description===
MixedAssignment fires when the right-hand side of an assignment resolves to mixed,
such as a mixed array element access.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @var array<string, mixed> $data */
$data = [];
$value = $data["key"];
//<^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $value is assigned a mixed type
