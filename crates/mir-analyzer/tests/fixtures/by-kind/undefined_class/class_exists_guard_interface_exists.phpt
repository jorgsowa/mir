===description===
interface_exists guard suppresses UndefinedClass in true branch
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    if (interface_exists(\Optional\Iface::class)) {
        $x = new class implements \Optional\Iface {};
    }
}
===expect===
