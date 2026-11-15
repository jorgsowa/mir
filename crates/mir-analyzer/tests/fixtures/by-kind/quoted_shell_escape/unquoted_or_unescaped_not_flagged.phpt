===description===
An `escapeshellarg()` result used on its own, a quoted value that isn't an
`escapeshellarg()` result, and a variable reassigned after escaping are fine.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function listDir(string $mode): void {
    system('ls ' . escapeshellarg($_GET['dir']));
    $dir = escapeshellarg($_GET['dir']);
    exec("ls -la $dir");
    system("ls '" . $mode . "'");
    passthru("tar -czf '/tmp/backup.tgz' " . escapeshellarg($_GET['dir']));
    $dir = escapeshellarg($_GET['dir']);
    $dir = 'logs';
    shell_exec("du -sh '" . $dir . "'");
}
