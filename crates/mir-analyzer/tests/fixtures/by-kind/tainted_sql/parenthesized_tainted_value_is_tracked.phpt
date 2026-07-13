===description===
A parenthesized tainted expression ((`$sql`)) still taints a SQL sink —
is_expr_tainted previously had no arm to unwrap Parenthesized, silently
breaking propagation through any parenthesized subexpression.
===config===
suppress=MixedArgument,MixedArrayAccess
===file===
<?php
function run_query(mysqli $db): void {
    mysqli_query($db, ($_GET['sql']));
}
===expect===
TaintedSql@3:4-3:37: Tainted SQL query — possible SQL injection
