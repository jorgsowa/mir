===description===
$pdo->query($sql) is a SQL sink, same as the procedural mysqli_query().
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function run_query(PDO $pdo): void {
    $pdo->query($_GET['sql']);
}
===expect===
TaintedSql@3:4-3:29: Tainted SQL query — possible SQL injection
