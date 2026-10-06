===description===
concatenated query is reported
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    $sql = 'SELECT * FROM users WHERE id = ' . $_GET['id'];
    mysqli_query($db, $sql);
//  ^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
