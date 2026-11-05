===description===
`FILTER_SANITIZE_ADD_SLASHES` escapes SQL, and `FILTER_SANITIZE_STRING` strips markup; neither clears the other kind.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function slashed(mysqli $db): void {
    $x = filter_var($_GET['x'], FILTER_SANITIZE_ADD_SLASHES);
    mysqli_query($db, "SELECT '$x'");
    echo $x;
//  ^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}

function stripped(mysqli $db): void {
    $x = filter_var($_GET['x'], FILTER_SANITIZE_STRING);
    echo $x;
    mysqli_query($db, "SELECT '$x'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
