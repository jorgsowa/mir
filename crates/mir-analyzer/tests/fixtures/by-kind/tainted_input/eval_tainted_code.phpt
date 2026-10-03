===description===
`eval($tainted)` was entirely unchecked — `eval` is its own AST node (not a
FunctionCall), so the taint-sink dispatch keyed on function names never saw
it. Tainted data reaching eval() is arbitrary code execution.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $code = $_GET['code'];
    eval($code);
//  ^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'eval'
}

function testSafe(): void {
    eval('1 + 1;');
}
===expect===
