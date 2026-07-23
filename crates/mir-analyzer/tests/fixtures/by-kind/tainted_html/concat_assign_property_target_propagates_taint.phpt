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
}
===expect===
TaintedHtml@8:4-8:17: Tainted HTML output — possible XSS
