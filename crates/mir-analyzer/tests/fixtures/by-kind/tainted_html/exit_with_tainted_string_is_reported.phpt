===description===
`exit`/`die` print a string operand, so it is an HTML sink.
===config===
suppress=MixedArrayAccess,MixedArgument
===file===
<?php
function test(): void {
    die($_GET['msg']);
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
