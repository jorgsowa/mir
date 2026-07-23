===description===
`+=` (and the rest of the arithmetic compound-assign family) never
propagated taint at all -- unlike `.=`'s already-fixed sticky taint, the
AssignOp::Plus|Minus|Mul|Div|Mod|Pow arm never called taint_var/taint_prop.
===config===
suppress=MixedAssignment,MixedArrayAccess,MixedArgument,MixedOperand
===file===
<?php
function test(): void {
    $id = 0;
    $id += $_GET['id'];
    echo $id;
}
===expect===
TaintedHtml@5:4-5:13: Tainted HTML output — possible XSS
