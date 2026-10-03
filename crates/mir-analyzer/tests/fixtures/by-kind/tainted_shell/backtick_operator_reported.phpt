===description===
The backtick shell-exec operator (`` `cmd` ``) is a sink exactly like its
functional twin `shell_exec()`/`exec()`, but this arm discarded its
interpolated parts entirely and never ran them through is_expr_tainted,
so a tainted value never produced TaintedShell here.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <ForbiddenCode errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $dir = $_GET['dir'];
    $out = `ls $dir`;
//         ^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
===expect===
