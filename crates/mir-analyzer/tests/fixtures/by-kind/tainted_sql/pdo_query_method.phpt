===description===
$pdo->query($sql) is a SQL sink, same as the procedural mysqli_query().
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(PDO $pdo): void {
    $pdo->query($_GET['sql']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
