===description===
FN: taint through an array literal was never tracked — `is_expr_tainted` had
no arm for `ExprKind::Array`, so `$arr = ['q' => $_GET['x']]; sink($arr['q']);`
went unreported even though `$arr` (and thus `$arr['q']`) is tainted.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    $arr = ['q' => $_GET['x']];
    mysqli_query($db, $arr['q']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
