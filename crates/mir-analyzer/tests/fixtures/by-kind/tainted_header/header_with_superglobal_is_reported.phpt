===description===
Tainted input in `header()` reports TaintedHeader, not TaintedHtml.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function redirect(): void {
    header('Location: ' . $_GET['next']);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ TaintedHeader: Tainted HTTP header — possible header injection or open redirect
}
