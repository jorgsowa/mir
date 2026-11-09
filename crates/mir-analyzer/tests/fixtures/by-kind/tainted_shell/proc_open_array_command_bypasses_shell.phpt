===description===
`proc_open()` with an array command runs the program without a shell; a string command is still a sink.
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
    $cmd = ['git', 'show', $_GET['ref']];
    proc_open($cmd, [], $pipes);
    proc_open('git log ' . $_GET['ref'], [], $pipes);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
