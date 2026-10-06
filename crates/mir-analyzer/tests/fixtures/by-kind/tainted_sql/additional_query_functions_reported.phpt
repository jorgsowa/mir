===description===
Prepare/exec variants across database extensions are SQL sinks.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingParamType errorLevel="suppress"/>
    <UndefinedFunction errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(mysqli $db, $odbc): void {
    mysqli_prepare($db, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    odbc_exec($odbc, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    oci_parse($odbc, $_GET['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
