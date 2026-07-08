===description===
FN: taint through a `match` expression arm was never tracked — `is_expr_tainted`
had no arm for `ExprKind::Match`, so a tainted value returned from a match arm
reaching a SQL sink went unreported.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment
===file===
<?php
function run_query(mysqli $db, int $mode): void {
    $sql = match ($mode) {
        1 => $_GET['sql'],
        default => 'SELECT 1',
    };
    mysqli_query($db, $sql);
}
===expect===
TaintedSql@7:4-7:27: Tainted SQL query — possible SQL injection
