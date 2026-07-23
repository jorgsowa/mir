===description===
`??=` never propagated taint at all -- the AssignOp::Coalesce arm computed
the merged type but never called taint_var, even though the equivalent
`$name = $name ?? $_GET['name'];` was already correctly tracked via
taint.rs's NullCoalesce arm.
===config===
suppress=MixedAssignment,MixedArrayAccess
===file===
<?php
function test(): void {
    $name = null;
    $name ??= $_GET['name'];
    echo $name;
}
===expect===
TaintedHtml@5:4-5:15: Tainted HTML output — possible XSS
