===description===
Static properties were entirely untracked for taint -- no write-side
taint arm, no read-side StaticPropertyAccess arm in is_expr_tainted, no
static-keyed taint set in FlowState at all (unlike instance properties,
which already had this via tainted_props).
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Registry {
    public static $lastQuery;
}

function run(mysqli $db): void {
    Registry::$lastQuery = $_GET['q'];
    mysqli_query($db, Registry::$lastQuery);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedSql: Tainted SQL query — possible SQL injection
}
===expect===
