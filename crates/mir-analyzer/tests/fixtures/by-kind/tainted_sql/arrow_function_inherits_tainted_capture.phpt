===description===
An arrow function auto-captures outer variables by value, including their
taint status — a tainted value flowing into a sink through
fn() => sink($tainted) was previously never flagged, unlike the equivalent
use($tainted) closure.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    $tainted = $_GET['sql'];
//  ^^^^^^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $tainted is assigned a mixed type
    $f = fn() => mysqli_query($db, $tainted);
//               ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
    $f();
}
