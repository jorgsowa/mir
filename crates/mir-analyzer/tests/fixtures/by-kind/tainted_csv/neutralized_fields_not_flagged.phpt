===description===
Fields with a safe literal prefix, numeric fields and program data can't become
formulas.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @param resource $out
 * @param list<string> $row
 */
function export($out, array $row, string $label): void {
    fputcsv($out, ["'" . $_POST['name'], (int) $_POST['age'], 'Note: ' . $_POST['note']]);
    fputcsv($out, $row);
    fputcsv($out, [$label, 'total']);
}
