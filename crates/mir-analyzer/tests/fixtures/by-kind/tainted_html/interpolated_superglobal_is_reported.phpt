===description===
interpolated superglobal is reported
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function render(): void {
    echo "Hello {$_GET['name']}";
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
