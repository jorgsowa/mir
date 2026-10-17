===description===
A tainted variable narrowed to numbers, booleans or program-defined string
literals can't carry an injection payload, so sinks don't report it.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function numeric(mysqli $db): void {
    $id = $_GET['id'];
    if (is_numeric($id)) {
        mysqli_query($db, "SELECT * FROM t WHERE id = " . $id);
    }
    mysqli_query($db, "SELECT * FROM t WHERE id = " . $id);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}

function allowList(mysqli $db): void {
    $dir = $_GET['dir'];
    if (in_array($dir, ['ASC', 'DESC'], true)) {
        mysqli_query($db, "SELECT * FROM t ORDER BY id " . $dir);
    }
    if (in_array($dir, ['ASC', 'DESC'])) {
        mysqli_query($db, "SELECT * FROM t ORDER BY id " . $dir);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    }
}

function identity(mysqli $db): void {
    $mode = $_GET['mode'];
    if ($mode === 'full') {
        mysqli_query($db, "SELECT * FROM t WHERE mode = '" . $mode . "'");
    }
}
