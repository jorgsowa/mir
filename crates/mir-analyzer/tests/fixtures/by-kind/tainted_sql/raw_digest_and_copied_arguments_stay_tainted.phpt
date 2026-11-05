===description===
Raw digests can hold quotes, and `uniqid`'s prefix and `number_format`'s separators are copied into the result. The hex forms and the formatted number stay clean.
===config===
<mir>
  <issueHandlers>
    <MissingParamType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedMethodCall errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
    <UndefinedClass errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function raw(mysqli $db): void {
    mysqli_query($db, "SELECT '" . md5($_GET['x'], true) . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    mysqli_query($db, "SELECT '" . hash('sha256', $_GET['x'], true) . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}

function hex(mysqli $db): void {
    mysqli_query($db, "SELECT '" . md5($_GET['x']) . "'");
    mysqli_query($db, "SELECT '" . hash('sha256', $_GET['x']) . "'");
    mysqli_query($db, "SELECT '" . number_format($_GET['x']) . "'");
}

function copied(mysqli $db): void {
    mysqli_query($db, "SELECT '" . uniqid($_GET['x']) . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    mysqli_query($db, "SELECT '" . number_format(1.5, 2, $_GET['x']) . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
