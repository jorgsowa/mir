===description===
$mysqli->query($sql) (OOP mysqli API) is a SQL sink, same as mysqli_query().
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function run_query(mysqli $db): void {
    $db->query($_GET['sql']);
}
===expect===
TaintedSql@3:4-3:28: Tainted SQL query — possible SQL injection
