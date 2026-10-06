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
//  ^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    mkdir($_GET['p']);
//  ^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    scandir($_GET['p']);
//  ^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    copy($_GET['p'], '/tmp/x');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    glob($_GET['p']);
//  ^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
    parse_ini_file($_GET['p']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedInput: Tainted input reaching sink 'file'
}
