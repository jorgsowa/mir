===description===
The backtick shell-exec operator (`` `cmd` ``) is a sink exactly like its
functional twin `shell_exec()`/`exec()`, but this arm discarded its
interpolated parts entirely and never ran them through is_expr_tainted,
so a tainted value never produced TaintedShell here.
===config===
suppress=MixedArgument,MixedAssignment,ForbiddenCode,MixedArrayAccess,UnusedVariable
===file===
<?php
function test(): void {
    $dir = $_GET['dir'];
    $out = `ls $dir`;
}
===expect===
TaintedShell@4:11-4:20: Tainted shell command — possible command injection
