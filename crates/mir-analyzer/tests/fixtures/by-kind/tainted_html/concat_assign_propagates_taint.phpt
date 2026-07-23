===description===
`.=` never propagated taint at all -- the AssignOp::Concat arm never called
taint_var/taint_prop, unlike the plain `=` arm right above it. This is one
of the most common HTML-building idioms.
===config===
suppress=MixedAssignment,MixedArrayAccess
===file===
<?php
function test(): void {
    $html = '<p>';
    $html .= $_GET['name'];
    echo $html;
}
===expect===
TaintedHtml@5:4-5:15: Tainted HTML output — possible XSS
