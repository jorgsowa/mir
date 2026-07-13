===description===
A custom class extending PDO is still a SQL sink through inheritance.
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
class AppDatabase extends PDO {}
function run_query(AppDatabase $pdo): void {
    $pdo->query($_GET['sql']);
}
===expect===
TaintedSql@4:4-4:29: Tainted SQL query — possible SQL injection
