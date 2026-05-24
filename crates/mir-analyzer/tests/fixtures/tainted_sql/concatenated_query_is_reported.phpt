===description===
concatenated query is reported
===file===
<?php
function run_query(mysqli $db): void {
    $sql = 'SELECT * FROM users WHERE id = ' . $_GET['id'];
    mysqli_query($db, $sql);
}
===expect===
TaintedSql@4:5: Tainted SQL query — possible SQL injection
