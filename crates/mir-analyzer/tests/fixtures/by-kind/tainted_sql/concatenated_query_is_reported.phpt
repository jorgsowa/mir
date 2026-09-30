===description===
concatenated query is reported
===config===
suppress=MixedArrayAccess
===file===
<?php
function run_query(mysqli $db): void {
    $sql = 'SELECT * FROM users WHERE id = ' . $_GET['id'];
    mysqli_query($db, $sql);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
