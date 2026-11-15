===description===
Wrapping an `escapeshellarg()` result in another pair of quotes closes them, so the
value runs unquoted in single quotes and still expands `$(...)` in double quotes.
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
function listDir(): void {
    system("ls '" . escapeshellarg($_GET['dir']) . "'");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ QuotedShellEscape: escapeshellarg() result placed inside quotes — its own quotes close them, so the value runs unquoted
}

function search(): void {
    $pattern = escapeshellarg($_GET['q']);
    exec("grep \"$pattern\" /var/log/app.log");
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ QuotedShellEscape: escapeshellarg() result placed inside quotes — its own quotes close them, so the value runs unquoted
}

function viaBackticks(): void {
    $file = escapeshellarg($_GET['file']);
    $lines = `wc -l '$file'`;
//           ^^^^^^^^^^^^^^^ QuotedShellEscape: escapeshellarg() result placed inside quotes — its own quotes close them, so the value runs unquoted
}
