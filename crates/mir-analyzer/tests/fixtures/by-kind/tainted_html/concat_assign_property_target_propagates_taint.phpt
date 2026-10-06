===description===
`.=` on a property target (`$b->log .= $tainted`) went through the
non-variable branch of AssignOp::Concat, which analyzed/reassigned the
target's type but never propagated taint either.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Logger {
    public $log = '';
}
function test(): void {
    $b = new Logger();
    $b->log .= $_GET['msg'];
    echo $b->log;
//  ^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
