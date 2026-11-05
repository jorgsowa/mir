===description===
`is_numeric`, like `ctype_digit`, clears a checked array element and not just a variable.
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
function checked(): void {
    $o = explode('.', $_GET['ip']);
    if (is_numeric($o[0]) && is_numeric($o[1])) {
        shell_exec('ping ' . $o[0] . '.' . $o[1]);
    }
}

function otherElement(): void {
    $o = explode('.', $_GET['ip']);
    if (is_numeric($o[0])) {
        shell_exec('ping ' . $o[1]);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
    }
}
