===description===
FP: `(int)`/`(float)`/`(bool)` casts coerce the value to that scalar type at
runtime, which sanitizes it against SQL/shell/HTML injection — but
`is_expr_tainted` ignored the cast kind and kept propagating taint through
any cast, including the standard `(int) $_GET['id']` defensive idiom.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <ImplicitToStringCast errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run_query(mysqli $db): void {
    $id = (int) $_GET['id'];
    mysqli_query($db, "SELECT * FROM t WHERE id = $id");
}
===expect===
