===description===
$mysqli->query($sql) (OOP mysqli API) is a SQL sink, same as mysqli_query().
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    $db->query($_GET['sql']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
