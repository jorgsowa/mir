===description===
`.=`'s result keeps the OLD value's content, unlike plain `=` which fully
replaces it -- appending a clean literal to an already-tainted variable
must not clear its taint.
===config===
suppress=MixedAssignment,MixedArrayAccess
===file===
<?php
function test(): void {
    $html = $_GET['name'];
    $html .= '</p>';
    echo $html;
}
===expect===
TaintedHtml@5:4-5:15: Tainted HTML output — possible XSS
