===description===
basic
===file===
<?php
function run_query(mysqli $db): void {
    mysqli_query($db, $_GET['sql']);
}
===expect===
TaintedSql@3:5: Tainted SQL query — possible SQL injection
