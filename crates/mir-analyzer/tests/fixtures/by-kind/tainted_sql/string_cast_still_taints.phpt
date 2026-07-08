===description===
A `(string)` cast does not remove the injection payload, unlike `(int)`/
`(float)`/`(bool)` — taint must still propagate through it.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
function run_query(mysqli $db): void {
    $sql = (string) $_GET['sql'];
    mysqli_query($db, $sql);
}
===expect===
TaintedSql@4:4-4:27: Tainted SQL query — possible SQL injection
