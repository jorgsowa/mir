===description===
`.=`'s result keeps the OLD value's content, unlike plain `=` which fully
replaces it -- appending a clean literal to an already-tainted variable
must not clear its taint.
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
    $html = $_GET['name'];
    $html .= '</p>';
    echo $html;
//  ^^^^^^^^^^^ TaintedHtml: Tainted HTML output — possible XSS
}
===expect===
