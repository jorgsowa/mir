===description===
FN: taint through an array literal was never tracked — `is_expr_tainted` had
no arm for `ExprKind::Array`, so `$arr = ['q' => $_GET['x']]; sink($arr['q']);`
went unreported even though `$arr` (and thus `$arr['q']`) is tainted.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
function run_query(mysqli $db): void {
    $arr = ['q' => $_GET['x']];
    mysqli_query($db, $arr['q']);
}
===expect===
TaintedSql@4:4-4:32: Tainted SQL query — possible SQL injection
