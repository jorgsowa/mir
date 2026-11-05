===description===
A builtin sink named as a callback receives the data the callback builtin hands it.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function direct(): void {
    call_user_func('system', $_GET['x']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}

function mapped(): void {
    array_map('shell_exec', $_GET);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}

function clean(): void {
    call_user_func('system', 'ls');
    array_map('strtolower', $_GET);
}
