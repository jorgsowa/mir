===description===
a constant path is never a taint sink
===config===
<mir>
  <issueHandlers>
    <MixedArgument errorLevel="suppress"/>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $data = file_get_contents('/etc/hostname');
    $obj = unserialize('a:0:{}');
}
