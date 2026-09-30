===description===
`.=` on a property target (`$b->log .= $tainted`) went through the
non-variable branch of AssignOp::Concat, which analyzed/reassigned the
target's type but never propagated taint either.
===config===
suppress=MixedAssignment,MissingConstructor,MixedArrayAccess,MissingPropertyType
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
===expect===
