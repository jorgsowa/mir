===description===
Static properties were entirely untracked for taint -- no write-side
taint arm, no read-side StaticPropertyAccess arm in is_expr_tainted, no
static-keyed taint set in FlowState at all (unlike instance properties,
which already had this via tainted_props).
===config===
suppress=UnusedParam,MixedArrayAccess,MissingPropertyType,MixedArgument
===file===
<?php
class Registry {
    public static $lastQuery;
}

function run(mysqli $db): void {
    Registry::$lastQuery = $_GET['q'];
    mysqli_query($db, Registry::$lastQuery);
}
===expect===
TaintedSql@8:4-8:43: Tainted SQL query — possible SQL injection
