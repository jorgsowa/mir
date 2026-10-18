===description===
Constant paths, and tainted values in non-path positions (written data, a
mode), are not reported.
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
    mkdir('/tmp/x');
    copy('/tmp/a', '/tmp/b');
    file_put_contents('/tmp/a', $_GET['data']);
    chmod('/tmp/a', 0644);
}
