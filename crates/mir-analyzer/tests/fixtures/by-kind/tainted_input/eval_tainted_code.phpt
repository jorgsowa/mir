===description===
`eval($tainted)` was entirely unchecked — `eval` is its own AST node (not a
FunctionCall), so the taint-sink dispatch keyed on function names never saw
it. Tainted data reaching eval() is arbitrary code execution.
===config===
suppress=MixedArgument,MixedArrayAccess,MixedAssignment,UnusedVariable
===file===
<?php
function test(): void {
    $code = $_GET['code'];
    eval($code);
}

function testSafe(): void {
    eval('1 + 1;');
}
===expect===
TaintedInput@4:4-4:15: Tainted input reaching sink 'eval'
