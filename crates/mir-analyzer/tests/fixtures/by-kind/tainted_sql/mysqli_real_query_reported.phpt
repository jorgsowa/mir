===description===
`mysqli_real_query()` and `mysqli_multi_query()` are SQL sinks.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function test(mysqli $db): void {
    mysqli_real_query($db, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    mysqli_multi_query($db, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
