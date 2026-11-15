===description===
A request value naming the file `CURLFile` or `curl_file_create()` attaches makes
the server upload any local file it can read.
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
function attach(): void {
    $file = new CURLFile($_POST['path']);
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    $other = curl_file_create($_GET['path'], 'text/plain');
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    $safe = new CURLFile('/var/app/report.pdf', 'application/pdf', (string) $_POST['name']);
}
