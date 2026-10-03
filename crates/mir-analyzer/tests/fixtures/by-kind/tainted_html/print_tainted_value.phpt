===description===
print($_GET['x']) is an HTML sink, same as echo $_GET['x'].
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function test(): void {
    print($_GET['x']);
//  ^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
