===description===
`mysqli_real_query()` and `mysqli_multi_query()` are SQL sinks.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(mysqli $db): void {
    mysqli_real_query($db, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    mysqli_multi_query($db, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
