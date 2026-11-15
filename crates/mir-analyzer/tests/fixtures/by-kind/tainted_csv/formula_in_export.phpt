===description===
Request data written as a CSV field can start with `=`, `+`, `-` or `@`, which a
spreadsheet runs as a formula.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param resource $out */
function export($out, SplFileObject $file): void {
    fputcsv($out, [$_POST['name'], $_POST['email']]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCsv: Tainted CSV field — possible spreadsheet formula injection
    $file->fputcsv(['id', '=' . $_GET['note']]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCsv: Tainted CSV field — possible spreadsheet formula injection
    fputcsv($out, $_POST);
//  ^^^^^^^^^^^^^^^^^^^^^ TaintedCsv: Tainted CSV field — possible spreadsheet formula injection
}
