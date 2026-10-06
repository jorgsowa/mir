===description===
`exit`/`die` print a string operand, so it is an HTML sink.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
    <MixedArgument errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    die($_GET['msg']);
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
