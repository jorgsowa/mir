===description===
Filesystem functions besides fopen/file_get_contents treat their path as a File sink.
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    unlink($_GET['p']);
//  ^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    mkdir($_GET['p']);
//  ^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    scandir($_GET['p']);
//  ^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    copy($_GET['p'], '/tmp/x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    glob($_GET['p']);
//  ^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
    parse_ini_file($_GET['p']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedFile: Tainted file path — possible path traversal
}
