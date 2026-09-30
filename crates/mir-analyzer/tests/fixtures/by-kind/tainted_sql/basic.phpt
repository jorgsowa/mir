===description===
Basic
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function run_query(mysqli $db): void {
    mysqli_query($db, $_GET['sql']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
