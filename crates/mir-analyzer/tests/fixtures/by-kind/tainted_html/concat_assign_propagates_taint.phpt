===description===
`.=` never propagated taint at all -- the AssignOp::Concat arm never called
taint_var/taint_prop, unlike the plain `=` arm right above it. This is one
of the most common HTML-building idioms.
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
    $html = '<p>';
    $html .= $_GET['name'];
    echo $html;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
