===description===
unpack not reported
===config===
<mir>
  <issueHandlers>
    <ForbiddenCode errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $r = unpack('N*', pack('N*', 1));
    var_dump($r);
}
===expect===
