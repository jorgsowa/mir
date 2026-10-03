===description===
A parenthesized tainted expression ((`$sql`)) still taints a SQL sink —
is_expr_tainted previously had no arm to unwrap Parenthesized, silently
breaking propagation through any parenthesized subexpression.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    mysqli_query($db, ($_GET['sql']));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
