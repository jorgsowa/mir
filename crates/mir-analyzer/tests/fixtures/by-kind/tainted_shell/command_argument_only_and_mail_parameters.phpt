===description===
Only the command of a shell function is a sink, not the environment or
descriptors passed alongside it. `mail()`'s extra headers are a header sink
and its extra parameters reach sendmail's command line.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run(): void {
    proc_open('ls', [], $pipes, null, ['QUERY' => $_GET['q']]);
    exec('ls', $output);
    proc_open('ls ' . $_GET['dir'], [], $pipes);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}

function send(): void {
    mail('admin@example.com', 'Report', $_GET['body']);
    mail('admin@example.com', 'Report', 'Body', 'Reply-To: ' . $_GET['email']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
    mail('admin@example.com', 'Report', 'Body', '', '-f' . $_GET['from']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedShell: Tainted shell command — possible command injection
}
