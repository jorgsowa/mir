===description===
`??=` never propagated taint at all -- the AssignOp::Coalesce arm computed
the merged type but never called taint_var, even though the equivalent
`$name = $name ?? $_GET['name'];` was already correctly tracked via
taint.rs's NullCoalesce arm.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $name = null;
    $name ??= $_GET['name'];
    echo $name;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
