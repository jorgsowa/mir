===description===
An arrow function auto-captures outer variables by value, including their
taint status — a tainted value flowing into a sink through
fn() => sink($tainted) was previously never flagged, unlike the equivalent
use($tainted) closure.
===config===
suppress=MixedArgument,MixedArrayAccess,UnusedVariable
===file===
<?php
function run_query(mysqli $db): void {
    $tainted = $_GET['sql'];
    $f = fn() => mysqli_query($db, $tainted);
    $f();
}
===expect===
MixedAssignment@3:4-3:27: Variable $tainted is assigned a mixed type
TaintedSql@4:17-4:44: Tainted SQL query — possible SQL injection
