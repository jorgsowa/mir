===description===
`proc_open()` with an array command runs the program without a shell, so only a
string command is a shell sink; an argument that can start with `-` is reported as
argument injection instead.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    proc_open(['git', 'log', $_GET['ref']], [], $pipes);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedCommandArgument: Tainted command argument can start with '-' — possible argument injection
    $cmd = ['git', 'show', $_GET['ref']];
    proc_open($cmd, [], $pipes);
    proc_open('git log ' . $_GET['ref'], [], $pipes);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
