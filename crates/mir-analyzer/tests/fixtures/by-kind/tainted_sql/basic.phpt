===description===
Basic
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function run_query(mysqli $db): void {
    mysqli_query($db, $_GET['sql']);
}
===expect===
TaintedSql@3:5-3:36: Tainted SQL query — possible SQL injection
