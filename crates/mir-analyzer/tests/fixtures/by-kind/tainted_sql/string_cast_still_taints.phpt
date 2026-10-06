===description===
A `(string)` cast does not remove the injection payload, unlike `(int)`/
`(float)`/`(bool)` — taint must still propagate through it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    $sql = (string) $_GET['sql'];
    mysqli_query($db, $sql);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
