===description===
$pdo->query() with a constant query string is not flagged.
===file===
<?php
function run_query(PDO $pdo): void {
    $pdo->query("SELECT * FROM users");
}
===expect===
