===description===
Constant paths, and tainted values in non-path positions, are not reported.
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
    copy('/tmp/a', $_GET['dest']);
    chmod('/tmp/a', 0644);
}
