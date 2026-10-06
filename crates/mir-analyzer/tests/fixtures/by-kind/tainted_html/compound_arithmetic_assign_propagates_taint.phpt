===description===
`+=` (and the rest of the arithmetic compound-assign family) never
propagated taint at all -- unlike `.=`'s already-fixed sticky taint, the
AssignOp::Plus|Minus|Mul|Div|Mod|Pow arm never called taint_var/taint_prop.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
    <MixedOperand errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    $id = 0;
    $id += $_GET['id'];
    echo $id;
//  ^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
